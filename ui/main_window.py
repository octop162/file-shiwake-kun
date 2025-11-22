import tkinter as tk
from tkinter import ttk
from tkinterdnd2 import DND_FILES, TkinterDnD

class MainWindow(ttk.Frame):
    """
    The main window of the application, containing the drag-and-drop zone.
    """
    def __init__(self, master: TkinterDnD.Tk, on_file_drop: callable):
        super().__init__(master, padding="20")
        self.master = master
        self.on_file_drop = on_file_drop

        # --- Drop Zone ---
        self.drop_zone = ttk.Frame(self, style='Drop.TFrame', padding="20")
        self.drop_zone.pack(fill=tk.BOTH, expand=True)

        self.drop_instructions = ttk.Label(
            self.drop_zone, 
            text="ファイルをここにドラッグ＆ドロップ",
            style='Drop.TLabel'
        )
        self.drop_instructions.pack(pady=20, padx=20)
        
        self.hint_label = ttk.Label(
            self.drop_zone,
            text="Drag and drop files or folders here",
            style='Hint.TLabel'
        )
        self.hint_label.pack(pady=5)
        
        # --- Processing State Overlay ---
        self.processing_overlay = ttk.Frame(self.drop_zone, style='Processing.TFrame')
        # Overlay is not packed here; it will be placed on top when active

        self.spinner = ttk.Progressbar(self.processing_overlay, mode='indeterminate')
        self.spinner.pack(pady=10)
        
        ttk.Label(
            self.processing_overlay,
            text="処理中...",
            style='Processing.TLabel'
        ).pack(pady=10)

        # --- DND Setup ---
        # The DND_FILES type indicates that the widget can accept dropped files.
        self.drop_zone.drop_target_register(DND_FILES)
        self.drop_zone.dnd_bind('<<Drop>>', self.handle_drop)
        
        # --- Style Configuration ---
        self.style = ttk.Style(self)
        self.style.configure('Drop.TFrame', background='#eaf2ff', borderwidth=2, relief='dashed')
        self.style.configure('Drop.TLabel', background='#eaf2ff', font=('Helvetica', 14))
        self.style.configure('Hint.TLabel', background='#eaf2ff', foreground='#555555')
        self.style.configure('Processing.TFrame', background='#cccccc')
        self.style.configure('Processing.TLabel', background='#cccccc', font=('Helvetica', 12))


    def handle_drop(self, event):
        """
        Handles the file drop event.
        The `event.data` attribute contains a string of file paths,
        possibly enclosed in braces {} and separated by spaces.
        """
        # The data is a string of file paths, sometimes with braces.
        # We need to parse it carefully.
        path_string = event.data.strip()
        if path_string.startswith('{') and path_string.endswith('}'):
            path_string = path_string[1:-1]
        
        # Split by spaces, but handle paths with spaces by looking for drive letters or slashes
        # This is a simplification; a more robust solution might be needed for complex paths.
        files = []
        current_path = ''
        for part in path_string.split(' '):
            if ':/' in part or ':\\' in part:
                if current_path:
                    files.append(current_path.strip())
                current_path = part
            else:
                current_path += ' ' + part
        if current_path:
            files.append(current_path.strip())

        if files and self.on_file_drop:
            self.on_file_drop(files)

    def show_processing_state(self):
        """Displays the processing overlay and starts the spinner."""
        self.drop_instructions.pack_forget()
        self.hint_label.pack_forget()
        self.processing_overlay.place(relx=0.5, rely=0.5, anchor=tk.CENTER)
        self.spinner.start()

    def hide_processing_state(self):
        """Hides the processing overlay and stops the spinner."""
        self.spinner.stop()
        self.processing_overlay.place_forget()
        self.drop_instructions.pack(pady=20, padx=20)
        self.hint_label.pack(pady=5)


if __name__ == '__main__':
    # This requires a DND-aware Tk instance, so we use TkinterDnD.Tk()
    # To run this test, you need to have tkinterdnd2 installed:
    # pip install tkinterdnd2-universal
    
    # Dummy drop handler for testing
    def test_drop_handler(files):
        print("--- Files Dropped ---")
        for file in files:
            print(f"  - {file}")
        
        # Simulate processing
        main_window.show_processing_state()
        
        def finish_processing():
            print("--- Processing Finished ---")
            main_window.hide_processing_state()

        # Hide processing state after 2 seconds
        root.after(2000, finish_processing)

    # Use the special DND-enabled Tk root
    root = TkinterDnD.Tk()
    root.title("Main Window Test")
    root.geometry("600x400")

    main_window = MainWindow(root, on_file_drop=test_drop_handler)
    main_window.pack(fill=tk.BOTH, expand=True)
    
    # Add a button to test the processing state toggle
    def toggle_processing():
        if main_window.spinner.isrunning():
            main_window.hide_processing_state()
        else:
            main_window.show_processing_state()
            
    button = ttk.Button(root, text="Toggle Processing State", command=toggle_processing)
    button.pack(pady=10)

    root.mainloop()
