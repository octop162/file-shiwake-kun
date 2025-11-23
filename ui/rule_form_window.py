from tkinter import filedialog
import os
import tkinter as tk
from tkinter import ttk
from typing import Dict, Any, Optional, List
import datetime
import logging

logger = logging.getLogger(__name__)

class RuleFormWindow(tk.Toplevel):
    """
    A Toplevel window for adding or editing a single rule.
    """
    def __init__(self, master, rule: Optional[Dict[str, Any]] = None, on_submit: callable = None, on_close: callable = None):
        super().__init__(master)
        self.title("ルールの編集" if rule else "新規ルールの追加")
        window_width = 600
        window_height = 550
        self.geometry(f"{window_width}x{window_height}")
        self.on_close = on_close
        self.protocol("WM_DELETE_WINDOW", self._on_closing) # Set custom close handler

        # Calculate position for centering
        # Use master's geometry for centering relative to the main window
        master_x = master.winfo_x()
        master_y = master.winfo_y()
        master_width = master.winfo_width()
        master_height = master.winfo_height()

        x = master_x + (master_width // 2) - (window_width // 2)
        y = master_y + (master_height // 2) - (window_height // 2)

        self.geometry(f"+{x}+{y}") # Set window position

        if rule:
            logger.debug(f"Opening RuleFormWindow to edit rule: {rule.get('id')}")
        else:
            logger.debug("Opening RuleFormWindow to add new rule.")

        self.rule = rule or {}
        self.on_submit = on_submit
        
        self.name_var = tk.StringVar(value=self.rule.get('name', ''))
        self.operation_var = tk.StringVar(value=self.rule.get('operation', 'move'))
        
        initial_dest_pattern = self.rule.get('destination_pattern', '').replace('\\', '/')
        dest_path_part = ""
        dest_filename_part = ""

        if '{filename}' in initial_dest_pattern or '{extension}' in initial_dest_pattern:
            last_slash_idx = initial_dest_pattern.rfind('/')
            if last_slash_idx != -1:
                dest_path_part = initial_dest_pattern[:last_slash_idx]
                dest_filename_part = initial_dest_pattern[last_slash_idx + 1:]
            else: # No slash, so it's just a filename pattern
                dest_filename_part = initial_dest_pattern
        else: # No filename/extension variables, treat whole thing as path
            dest_path_part = initial_dest_pattern
            # If it's a new rule and no initial destination pattern, set default filename pattern
            if not self.rule and not initial_dest_pattern:
                dest_filename_part = "{filename}.{extension}"
            
        self.dest_path_var = tk.StringVar(value=dest_path_part)
        self.dest_filename_var = tk.StringVar(value=dest_filename_part)
        
        # For now, conditions are not editable in this simple form
        self.conditions = self.rule.get('conditions', [])

        self.create_widgets()

    def _on_closing(self):
        """Handles the window closing event."""
        logger.debug("RuleFormWindow is closing.")
        if self.on_close:
            self.on_close()
        self.destroy()

    def create_widgets(self):
        main_frame = ttk.Frame(self, padding="10")
        main_frame.pack(fill=tk.BOTH, expand=True)

        # --- Basic Rule Info ---
        info_frame = ttk.LabelFrame(main_frame, text="基本情報", padding="10")
        info_frame.pack(fill=tk.X, pady=5)
        
        ttk.Label(info_frame, text="ルール名:").grid(row=0, column=0, sticky=tk.W, pady=5)
        ttk.Entry(info_frame, textvariable=self.name_var).grid(row=0, column=1, columnspan=2, sticky=tk.EW, pady=5)

        ttk.Label(info_frame, text="操作:").grid(row=1, column=0, sticky=tk.W, pady=5)
        op_combo = ttk.Combobox(info_frame, textvariable=self.operation_var, values=['move', 'copy'], state='readonly')
        op_combo.grid(row=1, column=1, columnspan=2, sticky=tk.W, pady=5)

        ttk.Label(info_frame, text="移動先ディレクトリパターン:").grid(row=2, column=0, sticky=tk.W, pady=5)
        ttk.Entry(info_frame, textvariable=self.dest_path_var).grid(row=2, column=1, sticky=tk.EW, pady=5)
        ttk.Button(info_frame, text="参照...", command=self._browse_directory).grid(row=2, column=2, sticky=tk.W, padx=(5,0))
        
        ttk.Label(info_frame, text="移動先ファイル名パターン:").grid(row=3, column=0, sticky=tk.W, pady=5)
        ttk.Entry(info_frame, textvariable=self.dest_filename_var).grid(row=3, column=1, columnspan=2, sticky=tk.EW, pady=5)
        
        info_frame.columnconfigure(1, weight=1)

        # --- Variables Hint ---
        hint_frame = ttk.LabelFrame(main_frame, text="利用可能な変数", padding="10")
        hint_frame.pack(fill=tk.X, pady=10)
        
        variables_hint_text = (
            "変数は、移動先パスの中でファイルのメタデータに置き換えられます。\n\n"
            "主な変数:\n"
            "  • {year}: 年 (例: 2023)\n"
            "  • {month}: 月 (例: 09)\n"
            "  • {day}: 日 (例: 05)\n"
            "  • {filename}: ファイル名 (拡張子なし)\n"
            "  • {extension}: 拡張子 (ドットなし)\n"
            "  • {camera}: カメラモデル\n\n"
            "使用例:\n"
            "  D:/写真/{year}/{month}/{filename}.{extension}"
        )
        ttk.Label(hint_frame, text=variables_hint_text, justify=tk.LEFT).pack(anchor=tk.W)

        # --- Action Buttons ---
        action_frame = ttk.Frame(main_frame)
        action_frame.pack(fill=tk.X, pady=(20, 0), side=tk.BOTTOM)
        
        ttk.Button(action_frame, text="保存", command=self.submit).pack(side=tk.RIGHT)
        ttk.Button(action_frame, text="キャンセル", command=self._on_closing).pack(side=tk.RIGHT, padx=5)

    def _browse_directory(self):
        """Opens a dialog to choose a directory and updates the entry field."""
        # Temporarily release grab from the Toplevel window to allow filedialog to open correctly
        # and then re-grab focus after the dialog is closed.
        self.grab_release() # Release window grab
        directory = filedialog.askdirectory(title="移動先フォルダを選択")
        if directory:
            # Append a separator so the user can immediately add variables
            # Ensure forward slashes are used for consistency with patterns
            standardized_directory = directory.replace('\\', '/')
            self.dest_path_var.set(standardized_directory + '/')
        
        # Bring the Toplevel window back to focus and grab all events
        self.lift()
        self.focus_force()
        self.grab_set() # Re-grab window events

    def submit(self):
        logger.debug("Submit button clicked in RuleFormWindow.")
        if not self.on_submit:
            self.destroy()
            return
            
        combined_dest_pattern = ""
        path_part = self.dest_path_var.get()
        filename_part = self.dest_filename_var.get()

        if path_part:
            combined_dest_pattern = path_part
            if not combined_dest_pattern.endswith('/'):
                combined_dest_pattern += '/'
            combined_dest_pattern += filename_part
        else:
            combined_dest_pattern = filename_part

        updated_rule = {
            'id': self.rule.get('id') or f"rule-{int(datetime.datetime.now().timestamp())}",
            'name': self.name_var.get(),
            'priority': self.rule.get('priority', 99), # Keep priority internally for now
            'operation': self.operation_var.get(),
            'destination_pattern': combined_dest_pattern,
            'conditions': self.conditions
        }
        
        logger.debug(f"Calling on_submit with rule data: {updated_rule}")
        self.on_submit(updated_rule)
        self._on_closing()

if __name__ == '__main__':
    # Simple test for the RuleFormWindow
    import datetime

    class TestApp(tk.Tk):
        def __init__(self):
            super().__init__()
            self.title("Rule Form Test")
            
            def show_result(rule):
                print("--- Rule Submitted ---")
                import json
                print(json.dumps(rule, indent=2))

            ttk.Button(self, text="Add New Rule", command=lambda: RuleFormWindow(self, on_submit=show_result)).pack(pady=10)
            
            sample_rule = {
                "id": "rule-001", "name": "Sort Photos", "priority": 1,
                "operation": "move", "destination_pattern": "D:/Photos/{year}/{month}",
                "conditions": [{"field": "extension", "operator": "in", "value": [".jpg", ".jpeg"]}]
            }
            ttk.Button(self, text="Edit Existing Rule", command=lambda: RuleFormWindow(self, rule=sample_rule, on_submit=show_result)).pack(pady=10)

    app = TestApp()
    app.mainloop()
