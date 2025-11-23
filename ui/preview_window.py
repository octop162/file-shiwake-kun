import os
import tkinter as tk
from tkinter import ttk
from typing import List, Dict, Any

class PreviewWindow(tk.Toplevel):
    """
    A modal dialog to show the user what changes will be made and ask for confirmation.
    """
    def __init__(self, master, results: List[Dict[str, Any]]):
        super().__init__(master)
        self.title("プレビュー")
        window_width = 1000
        window_height = 600
        self.geometry(f"{window_width}x{window_height}")

        # Calculate position for centering relative to master
        master_x = master.winfo_x()
        master_y = master.winfo_y()
        master_width = master.winfo_width()
        master_height = master.winfo_height()

        x = master_x + (master_width // 2) - (window_width // 2)
        y = master_y + (master_height // 2) - (window_height // 2)

        self.geometry(f"+{x}+{y}") # Set window position

        self.results = results
        self.result = "cancel"  # Default action

        self.transient(master)
        self.grab_set()

        self.create_widgets()
        self.populate_results()

        self.wait_window(self)

    def create_widgets(self):
        main_frame = ttk.Frame(self, padding="10")
        main_frame.pack(fill=tk.BOTH, expand=True)

        summary_text = (f"以下のファイル操作が実行されます。よろしいですか？\n" 
                        f"合計 {len(self.results)}件の操作が予定されています。")
        
        summary_label = ttk.Label(main_frame, text=summary_text, font=('Helvetica', 12))
        summary_label.pack(fill=tk.X, pady=5)
        
        tree_frame = ttk.Frame(main_frame)
        tree_frame.pack(fill=tk.BOTH, expand=True, pady=5)
        
        self.tree = ttk.Treeview(
            tree_frame,
            columns=('operation', 'source_dir', 'source_file', 'dest_dir', 'dest_file', 'rule'),
            show='headings'
        )
        self.tree.heading('operation', text='操作')
        self.tree.heading('source_dir', text='元のディレクトリ')
        self.tree.heading('source_file', text='元のファイル名')
        self.tree.heading('dest_dir', text='移動先ディレクトリ')
        self.tree.heading('dest_file', text='移動先ファイル名')
        self.tree.heading('rule', text='適用ルール')

        self.tree.column('operation', width=80, anchor=tk.CENTER)
        self.tree.column('source_dir', width=200)
        self.tree.column('source_file', width=150)
        self.tree.column('dest_dir', width=200)
        self.tree.column('dest_file', width=150)
        self.tree.column('rule', width=120)

        scrollbar = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL, command=self.tree.yview)
        self.tree.configure(yscroll=scrollbar.set)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        
        action_frame = ttk.Frame(main_frame)
        action_frame.pack(fill=tk.X, pady=(10, 0))
        
        ttk.Button(action_frame, text="実行", command=lambda: self.resolve("confirm")).pack(side=tk.RIGHT)
        ttk.Button(action_frame, text="キャンセル", command=lambda: self.resolve("cancel")).pack(side=tk.RIGHT, padx=5)

    def populate_results(self):
        """Populates the Treeview with the proposed operations."""
        for plan in self.results:
            # The plan is a dict with 'file_path', 'rule', 'dest_path'
            rule = plan.get('rule')
            if not rule:
                continue

            source_path = plan.get('file_path')
            source_dir = os.path.dirname(source_path) if source_path else ''
            source_file = os.path.basename(source_path) if source_path else ''

            dest_path = plan.get('dest_path')
            dest_dir = os.path.dirname(dest_path) if dest_path else 'N/A'
            dest_file = os.path.basename(dest_path) if dest_path else 'N/A'

            self.tree.insert(
                '',
                tk.END,
                values=(
                    rule.get('operation'),
                    source_dir,
                    source_file,
                    dest_dir,
                    dest_file,
                    rule.get('name')
                )
            )
            
    def resolve(self, choice: str):
        """Sets the result and closes the dialog."""
        self.result = choice
        self.destroy()

if __name__ == '__main__':
    class TestApp(tk.Tk):
        def __init__(self):
            super().__init__()
            self.title("Preview Window Test")
            
            self.test_results = [
                {'source_path': 'C:/Users/test/img1.jpg', 'destination_path': 'D:/Photos/2023/img1.jpg', 'operation': 'move', 'success': True, 'matched_rule': 'Sort Photos', 'error_message': None},
                {'source_path': 'C:/Users/test/notes.txt', 'destination_path': None, 'operation': None, 'success': True, 'matched_rule': None, 'error_message': 'No matching rule found.'},
            ]
            
            ttk.Button(self, text="Show Preview", command=self.show_preview_win).pack(pady=50)

        def show_preview_win(self):
            dialog = PreviewWindow(self, self.test_results)
            print(f"User chose: {dialog.result}")

    app = TestApp()
    app.mainloop()
