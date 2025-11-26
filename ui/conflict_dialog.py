import os
import tkinter as tk
from tkinter import ttk, messagebox
from typing import Dict, Any

class ConflictDialog(tk.Toplevel):
    """
    A modal dialog to resolve a file name conflict.
    """
    def __init__(self, master, source_path: str, dest_path: str):
        super().__init__(master)
        self.title("ファイル名の競合")
        window_width = 600
        window_height = 300
        self.geometry(f"{window_width}x{window_height}")

        # Calculate position for centering relative to master
        master_x = master.winfo_x()
        master_y = master.winfo_y()
        master_width = master.winfo_width()
        master_height = master.winfo_height()

        x = master_x + (master_width // 2) - (window_width // 2)
        y = master_y + (master_height // 2) - (window_height // 2)

        self.geometry(f"+{x}+{y}") # Set window position

        self.source_path = source_path
        self.dest_path = dest_path
        self.result = {"resolution": "skip", "apply_to_all": False}  # Default action
        self.apply_to_all_var = tk.BooleanVar(value=False)

        self.transient(master)
        self.grab_set()
        
        self.create_widgets()

        # This makes the dialog modal
        self.wait_window(self)

    def create_widgets(self):
        main_frame = ttk.Frame(self, padding="15")
        main_frame.pack(fill=tk.BOTH, expand=True)

        message = (f"移動先に同名のファイルが既に存在します:\n" 
                   f"'{self.dest_path}'\n\n" 
                   f"どのように処理しますか？")
        
        ttk.Label(main_frame, text=message, wraplength=550).pack(fill=tk.X, pady=10)

        # In a real implementation, you might show file size/dates here

        check_frame = ttk.Frame(main_frame)
        check_frame.pack(fill=tk.X, pady=10)
        ttk.Checkbutton(
            check_frame,
            text="以降の競合では常にこの選択を適用する",
            variable=self.apply_to_all_var
        ).pack(side=tk.LEFT)
        
        button_frame = ttk.Frame(main_frame)
        button_frame.pack(pady=20)

        ttk.Button(button_frame, text="上書き", command=lambda: self.resolve("overwrite")).pack(side=tk.LEFT, padx=10)
        ttk.Button(button_frame, text="スキップ", command=lambda: self.resolve("skip")).pack(side=tk.LEFT, padx=10)
        ttk.Button(button_frame, text="名前を変更して保存", command=lambda: self.resolve("rename")).pack(side=tk.LEFT, padx=10)

    def resolve(self, choice: str):
        """Sets the result and closes the dialog."""
        resolution = choice
        if choice == "rename":
            base, ext = os.path.splitext(self.dest_path)
            resolution = f"{base}_1{ext}"
        self.result = {"resolution": resolution, "apply_to_all": self.apply_to_all_var.get()}
        self.destroy()

if __name__ == '__main__':
    import os

    class TestApp(tk.Tk):
        def __init__(self):
            super().__init__()
            self.title("Conflict Dialog Test")
            
            ttk.Button(self, text="Show Conflict Dialog", command=self.show_dialog).pack(pady=50)

        def show_dialog(self):
            source = "/path/to/source/file.txt"
            dest = "/path/to/destination/file.txt"
            dialog = ConflictDialog(self, source, dest)
            
            print(f"User chose: {dialog.result}")
            if dialog.result not in ["overwrite", "skip"]:
                print(f"New destination path: {dialog.result}")


    app = TestApp()
    app.mainloop()
