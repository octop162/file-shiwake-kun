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
        self.geometry("1000x600")

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
            columns=('operation', 'source', 'destination', 'rule'),
            show='headings'
        )
        self.tree.heading('operation', text='操作')
        self.tree.heading('source', text='元のパス')
        self.tree.heading('destination', text='移動先のパス')
        self.tree.heading('rule', text='適用ルール')

        self.tree.column('operation', width=80, anchor=tk.CENTER)
        self.tree.column('source', width=350)
        self.tree.column('destination', width=350)
        self.tree.column('rule', width=150)

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
        for result in self.results:
            # Only show files that have a matched rule
            if not result['matched_rule']:
                continue

            self.tree.insert(
                '',
                tk.END,
                values=(
                    result['operation'],
                    result['source_path'],
                    result['destination_path'] or 'N/A',
                    result['matched_rule']
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
