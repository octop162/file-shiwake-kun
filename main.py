import tkinter as tk
from tkinter import ttk, messagebox
from tkinterdnd2 import TkinterDnD
from ttkthemes import ThemedStyle
import logging

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
    """
    Main application class that ties the UI and logic together.
    """
    def __init__(self):
        super().__init__()

        # Apply theme using composition
        style = ThemedStyle(self)
        style.set_theme("arc")

        self.title("ファイル仕訳け君")
        self.geometry("900x750")

        # Load configuration
        try:
            self.config_manager = ConfigManager('config.json')
            self.config = self.config_manager.load_config()
        except Exception as e:
            messagebox.showerror("設定エラー", f"設定ファイルの読み込みに失敗しました。\n{e}")
            self.destroy()
            return

        # Initialize business logic components
        self.file_processor = FileProcessor(self.config, conflict_handler=self.handle_conflict)

        # Setup main view UI
        self.main_view = MainView(
            self, 
            config=self.config,
            on_save=self.save_config,
            on_file_drop=self.handle_file_drop
        )
        self.main_view.pack(fill=tk.BOTH, expand=True)

    def save_config(self, new_config):
        """Callback to save the configuration and update the app."""
        logging.debug(f"Saving new configuration: {new_config}")
        self.config = new_config
        self.config_manager.save_config(self.config)
        # Re-initialize the file processor with the new rules and handler
        self.file_processor = FileProcessor(self.config, conflict_handler=self.handle_conflict)
        logging.info("Configuration saved and processor updated.")

    def handle_conflict(self, source_path: str, dest_path: str) -> str:
        """Opens a dialog to resolve a file conflict and returns the user's choice."""
        dialog = ConflictDialog(self, source_path, dest_path)
        return dialog.result

    def handle_file_drop(self, file_paths: list[str]):
        """
        Callback function for when files are dropped on the main window.
        Handles the preview and final processing logic.
        """
        logging.info(f"Files dropped, starting processing for: {file_paths}")
        
        # In a real app, this would run in a thread, and we'd show a proper spinner
        # For now, the UI will freeze during processing.

        # Always run in preview mode first if the setting is enabled
        if self.config.get('preview_mode', False):
            preview_results = self.file_processor.process_files(file_paths)
            
            preview_dialog = PreviewWindow(self, preview_results)
            
            if preview_dialog.result == "confirm":
                logging.info("Preview confirmed. Running actual file operations...")
                temp_config = self.config.copy()
                temp_config['preview_mode'] = False
                final_processor = FileProcessor(temp_config, conflict_handler=self.handle_conflict)
                final_results = final_processor.process_files(file_paths)
                ResultsView(self, final_results)
            else:
                logging.info("Preview cancelled.")
        else:
            # If preview mode is off, just process directly
            results = self.file_processor.process_files(file_paths)
            ResultsView(self, results)

if __name__ == "__main__":
    app = Application()
    app.mainloop()