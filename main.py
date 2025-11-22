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

from ui.main_window import MainWindow
from ui.settings_window import SettingsWindow
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
        self.geometry("800x600")

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

        # Setup top-level controls
        self._create_controls()

        # Setup main window UI
        self.main_window = MainWindow(self, on_file_drop=self.handle_file_drop)
        self.main_window.pack(fill=tk.BOTH, expand=True)


    def _create_controls(self):
        """Creates top-level controls like a menu or a button bar."""
        control_frame = ttk.Frame(self, padding=(10, 10, 10, 0))
        control_frame.pack(fill=tk.X)

        settings_button = ttk.Button(
            control_frame,
            text="設定",
            command=self.open_settings
        )
        settings_button.pack(side=tk.LEFT)

    def open_settings(self):
        """Opens the settings window."""
        SettingsWindow(self, self.config, on_save=self.save_config)

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
        print("Files dropped, starting processing...")
        self.main_window.show_processing_state()

        # Always run in preview mode first if the setting is enabled
        if self.config.get('preview_mode', False):
            # Run with the current config (which has preview_mode=True)
            preview_results = self.file_processor.process_files(file_paths)
            self.main_window.hide_processing_state()
            
            # Show preview window
            preview_dialog = PreviewWindow(self, preview_results)
            
            # If user confirms, run for real
            if preview_dialog.result == "confirm":
                print("Preview confirmed. Running actual file operations...")
                self.main_window.show_processing_state()
                
                # Create a temporary config with preview mode disabled
                temp_config = self.config.copy()
                temp_config['preview_mode'] = False
                final_processor = FileProcessor(temp_config, conflict_handler=self.handle_conflict)
                
                final_results = final_processor.process_files(file_paths)
                self.main_window.hide_processing_state()
                ResultsView(self, final_results)
            else:
                print("Preview cancelled.")
        else:
            # If preview mode is off, just process directly
            results = self.file_processor.process_files(file_paths)
            self.main_window.hide_processing_state()
            ResultsView(self, results)

    def _create_menu(self):
        # TODO: Implement a menu bar for settings, exit, etc.
        pass

if __name__ == "__main__":
    # Before running, ensure dependencies are installed:
    # pip install Pillow tkinterdnd2-universal
    
    app = Application()
    app.mainloop()