import tkinter as tk
from tkinter import ttk, messagebox
from tkinterdnd2 import TkinterDnD
from ttkthemes import ThemedStyle
import logging
import threading
import queue
import sys
import os
from typing import List, Dict, Any

def get_base_path():
    """ Get the base path for data files, for PyInstaller. """
    if getattr(sys, 'frozen', False):
        # The application is frozen (packaged)
        return os.path.dirname(sys.executable)
    else:
        # The application is running in a normal Python environment
        return os.path.dirname(os.path.abspath(__file__))

# --- Basic Logging Setup ---
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
)
# ---

from ui.main_view import MainView
from ui.preview_window import PreviewWindow
from ui.results_view import ResultsView
from ui.conflict_dialog import ConflictDialog
from logic.file_processor import FileProcessor
from data.config_manager import ConfigManager

class Application(TkinterDnD.Tk):
    def __init__(self):
        super().__init__()

        style = ThemedStyle(self)
        style.set_theme("breeze")

        self.title("ファイル仕分君")
        window_width = 900
        window_height = 750
        self.geometry(f"{window_width}x{window_height}")

        # Get screen width and height
        screen_width = self.winfo_screenwidth()
        screen_height = self.winfo_screenheight()

        # Calculate position for centering
        x = (screen_width // 2) - (window_width // 2)
        y = (screen_height // 2) - (window_height // 2)

        self.geometry(f"+{x}+{y}") # Set window position

        # --- Threading & Queue Setup ---
        self.result_queue = queue.Queue()
        self.conflict_queue = queue.Queue()
        self.conflict_event = threading.Event()
        self.after(100, self._check_queue) # Start polling the queue

        try:
            config_path = os.path.join(get_base_path(), 'config.json')
            self.config_manager = ConfigManager(config_path)
            self.config = self.config_manager.load_config()
        except Exception as e:
            messagebox.showerror("設定エラー", f"設定ファイルの読み込みに失敗しました。\n{e}")
            self.destroy()
            return

        self.file_processor = FileProcessor(self.config, conflict_handler=self.handle_conflict)

        self.main_view = MainView(
            self, 
            config=self.config,
            on_save=self.save_config,
            on_file_drop=self.handle_file_drop
        )
        self.main_view.pack(fill=tk.BOTH, expand=True)

        last_selected_id = self.config.get("last_selected_rule_id")
        if last_selected_id:
            self.main_view.select_rule_by_id(last_selected_id)
        
        self.protocol("WM_DELETE_WINDOW", self.on_close)

    def _check_queue(self):
        """Polls the queue for results from the background thread."""
        try:
            result_data = self.result_queue.get(block=False)

            if result_data.get("type") == "progress":
                current = result_data["current"]
                total = result_data["total"]
                phase = result_data.get("phase", "処理中")

                if total > 0:
                    progress_text = f"{phase}: {current} / {total} ファイル"
                else: # Total is unknown during file discovery
                    progress_text = f"{phase}: {current} ファイル発見"
                self.main_view.update_progress_text(progress_text)
            
            elif result_data.get("type") == "result":
                results = result_data["results"]
                is_preview = result_data["is_preview"]

                if is_preview:
                    file_paths = result_data["file_paths"]
                    selected_rule_id = result_data["selected_rule_id"]
                    self._handle_preview_result(results, file_paths, selected_rule_id)
                else:
                    self.main_view.show_processing_state(False)
                    ResultsView(self, results)
            
            elif result_data.get("type") == "request_conflict_resolution":
                source = result_data["source"]
                dest = result_data["dest"]
                self._show_conflict_dialog(source, dest)

        except queue.Empty:
            pass # Keep polling
        
        self.after(100, self._check_queue)

    def on_close(self):
        logging.debug("Close event detected, saving last selection.")
        selected_id = self.main_view.get_selected_rule_id()
        self.config["last_selected_rule_id"] = selected_id
        self.save_config(self.config)
        self.destroy()

    def save_config(self, new_config):
        logging.debug(f"Saving new configuration: {new_config}")
        self.config = new_config
        self.config_manager.save_config(self.config)
        self.file_processor = FileProcessor(self.config, conflict_handler=self.handle_conflict)
        logging.info("Configuration saved and processor updated.")

    def handle_conflict(self, source_path: str, dest_path: str) -> str:
        """
        Called from the worker thread when a conflict occurs.
        It asks the main thread to show a dialog and waits for the result.
        """
        self.conflict_event.clear()
        self.result_queue.put({
            "type": "request_conflict_resolution",
            "source": source_path,
            "dest": dest_path
        })
        self.conflict_event.wait() # Wait until the main thread sets the event
        return self.conflict_queue.get()

    def _show_conflict_dialog(self, source_path: str, dest_path: str):
        dialog = ConflictDialog(self, source_path, dest_path)
        self.conflict_queue.put(dialog.result)
        self.conflict_event.set() # Signal the worker thread to continue

    def handle_file_drop(self, file_paths: list[str]):
        if not file_paths: return
        
        selected_rule_id = self.main_view.get_selected_rule_id()
        if not selected_rule_id:
            messagebox.showwarning("ルール未選択", "処理を実行する前に、リストからルールを1つ選択してください。")
            return
            
        logging.info(f"Files dropped, starting background processing for {len(file_paths)} files.")
        self.main_view.show_processing_state(True)
        self.main_view.update_progress_text("ファイル分析中...") # 初期テキストを設定

        # Re-initialize FileProcessor to reset "apply to all" state for each job
        self.file_processor = FileProcessor(self.config, conflict_handler=self.handle_conflict)

        # Run file processing in a background thread
        thread = threading.Thread(
            target=self._process_files_thread,
            args=(file_paths, selected_rule_id)
        )
        thread.daemon = True
        thread.start()

    def _process_files_thread(self, file_paths: list[str], selected_rule_id: str):
        """Worker function to run in a background thread to discover operations."""
        
        # Define a flexible callback to put progress updates into the queue
        def progress_callback(current, total=0):
            phase = "ファイルリスト作成中"
            if total > 0:
                phase = "ファイル分析中"

            self.result_queue.put({
                "type": "progress", "current": current, "total": total, "phase": phase
            })

        # This part just discovers the plan
        planned_operations = self.file_processor.discover_operations(
            file_paths, selected_rule_id, progress_callback
        )
        
        self.result_queue.put({
            "type": "result", # Add a type to distinguish from progress
            "results": planned_operations,
            "is_preview": True,
            "file_paths": file_paths,
            "selected_rule_id": selected_rule_id
        })

    def _handle_preview_result(self, planned_operations, file_paths, selected_rule_id):
        """Shows the preview window and handles the confirmation."""
        self.main_view.show_processing_state(False)
        
        executable_plans = [op for op in planned_operations if not op.get('error')]
        if not executable_plans:
            self.main_view.update_progress_text("") # Clear progress text
            messagebox.showinfo("プレビュー", "選択されたルールにマッチするファイルはありませんでした。")
            return

        preview_dialog = PreviewWindow(self, executable_plans)
        if preview_dialog.result == "confirm":
            logging.info("Preview confirmed. Executing operations in background.")
            self.main_view.show_processing_state(True)
            self.main_view.update_progress_text("ファイル処理実行中...") # 実行フェーズの初期テキスト
            
            # Start another thread for the final execution, passing the plans
            thread = threading.Thread(
                target=self._execute_confirmed_thread,
                args=(executable_plans,)
            )
            thread.daemon = True
            thread.start()
        else:
            self.main_view.update_progress_text("") # Clear progress text
            logging.info("Preview cancelled by user.")

    def _execute_confirmed_thread(self, planned_operations: List[Dict[str, Any]]):
        """Worker thread for executing a list of planned operations."""
        results = []
        total_ops = len(planned_operations)
        for i, plan in enumerate(planned_operations):
            # Send progress for the execution phase
            self.result_queue.put({
                "type": "progress", "current": i + 1, "total": total_ops, "phase": "ファイル処理実行中"
            })

            result = self.file_processor.execute_operation(
                file_path=plan['file_path'],
                rule=plan['rule'],
                dest_path=plan['dest_path']
            )
            results.append(result)
        
        self.main_view.update_progress_text("") # Clear progress text on completion
        self.result_queue.put({"type": "result", "results": results, "is_preview": False})


if __name__ == "__main__":
    app = Application()
    app.mainloop()