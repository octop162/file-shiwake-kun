import os
import tkinter as tk
from tkinter import ttk
from typing import List, Dict, Any

class ResultsView(tk.Toplevel):
    """
    A Toplevel window to display the results of a file processing operation.
    """
    def __init__(self, master, results: List[Dict[str, Any]]):
        super().__init__(master)
        self.title("処理結果")
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
        
        self.transient(master)
        self.grab_set()

        self.create_widgets()
        self.populate_results()

    def create_widgets(self):
        main_frame = ttk.Frame(self, padding="10")
        main_frame.pack(fill=tk.BOTH, expand=True)

        # --- Summary ---
        success_count = sum(1 for r in self.results if r['success'])
        fail_count = len(self.results) - success_count
        summary_text = f"処理完了: 合計 {len(self.results)}件 (成功: {success_count}件, 失敗: {fail_count}件)"
        
        summary_label = ttk.Label(main_frame, text=summary_text, font=('Helvetica', 12))
        summary_label.pack(fill=tk.X, pady=5)
        
        # --- Results Tree ---
        tree_frame = ttk.Frame(main_frame)
        tree_frame.pack(fill=tk.BOTH, expand=True, pady=5)
        
        self.tree = ttk.Treeview(
            tree_frame,
            columns=('status', 'source_dir', 'source_file', 'dest_dir', 'dest_file', 'details'),
            show='headings'
        )
        self.tree.heading('status', text='状態')
        self.tree.heading('source_dir', text='元のディレクトリ')
        self.tree.heading('source_file', text='元のファイル名')
        self.tree.heading('dest_dir', text='移動先ディレクトリ')
        self.tree.heading('dest_file', text='移動先ファイル名')
        self.tree.heading('details', text='詳細')

        self.tree.column('status', width=80, anchor=tk.CENTER)
        self.tree.column('source_dir', width=200)
        self.tree.column('source_file', width=150)
        self.tree.column('dest_dir', width=200)
        self.tree.column('dest_file', width=150)
        self.tree.column('details', width=150)

        # Add a scrollbar
        scrollbar = ttk.Scrollbar(tree_frame, orient=tk.VERTICAL, command=self.tree.yview)
        self.tree.configure(yscroll=scrollbar.set)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        
        # --- Tag configuration for colors ---
        self.tree.tag_configure('success', foreground='green')
        self.tree.tag_configure('failure', foreground='red')

        # --- Action Buttons ---
        action_frame = ttk.Frame(main_frame)
        action_frame.pack(fill=tk.X, pady=(10, 0))
        
        ttk.Button(action_frame, text="閉じる", command=self.destroy).pack(side=tk.RIGHT)

    def populate_results(self):
        """Populates the Treeview with the processing results."""
        for result in self.results:
            status_text = "成功" if result['success'] else "失敗"
            tag = 'success' if result['success'] else 'failure'
            
            # If no rule matched, it's a success but with a specific message
            details = result['error_message'] or result['matched_rule'] or "処理なし"
            if not result['matched_rule'] and result['success']:
                details = "マッチするルールがありません"

            source_path = result['source_path']
            source_dir = os.path.dirname(source_path) if source_path else ''
            source_file = os.path.basename(source_path) if source_path else ''

            dest_path = result['destination_path']
            dest_dir = os.path.dirname(dest_path) if dest_path else 'N/A'
            dest_file = os.path.basename(dest_path) if dest_path else 'N/A'

            self.tree.insert(
                '',
                tk.END,
                values=(
                    status_text,
                    source_dir,
                    source_file,
                    dest_dir,
                    dest_file,
                    details
                ),
                tags=(tag,)
            )

if __name__ == '__main__':
    # Simple test for the ResultsView window
    class TestApp(tk.Tk):
        def __init__(self):
            super().__init__()
            self.title("Results View Test")
            
            self.test_results = [
                {'source_path': 'C:/Users/test/img1.jpg', 'destination_path': 'D:/Photos/2023/img1.jpg', 'success': True, 'matched_rule': 'Sort Photos', 'error_message': None},
                {'source_path': 'C:/Users/test/doc1.pdf', 'destination_path': None, 'success': False, 'matched_rule': None, 'error_message': 'Failed to copy file.'},
                {'source_path': 'C:/Users/test/notes.txt', 'destination_path': None, 'success': True, 'matched_rule': None, 'error_message': None},
            ]
            
            ttk.Button(self, text="Show Results", command=self.show_results_win).pack(pady=50)

        def show_results_win(self):
            ResultsView(self, self.test_results)

    app = TestApp()
    app.mainloop()
