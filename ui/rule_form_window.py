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
    def __init__(self, master, rule: Optional[Dict[str, Any]] = None, on_submit: callable = None):
        super().__init__(master)
        self.title("ルールの編集" if rule else "新規ルールの追加")
        self.geometry("600x550")

        if rule:
            logger.debug(f"Opening RuleFormWindow to edit rule: {rule.get('id')}")
        else:
            logger.debug("Opening RuleFormWindow to add new rule.")

        self.rule = rule or {}
        self.on_submit = on_submit
        
        self.name_var = tk.StringVar(value=self.rule.get('name', ''))
        self.operation_var = tk.StringVar(value=self.rule.get('operation', 'move'))
        self.dest_pattern_var = tk.StringVar(value=self.rule.get('destination_pattern', ''))
        
        # For now, conditions are not editable in this simple form
        self.conditions = self.rule.get('conditions', [])

        self.create_widgets()

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

        ttk.Label(info_frame, text="移動先パターン:").grid(row=2, column=0, sticky=tk.W, pady=5)
        ttk.Entry(info_frame, textvariable=self.dest_pattern_var).grid(row=2, column=1, sticky=tk.EW, pady=5)
        ttk.Button(info_frame, text="参照...", command=self._browse_directory).grid(row=2, column=2, sticky=tk.W, padx=(5,0))
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
        ttk.Button(action_frame, text="キャンセル", command=self.destroy).pack(side=tk.RIGHT, padx=5)

    def _browse_directory(self):
        """Opens a dialog to choose a directory and updates the entry field."""
        directory = filedialog.askdirectory(title="移動先フォルダを選択")
        if directory:
            # Append a separator so the user can immediately add variables
            self.dest_pattern_var.set(directory + os.path.sep)

    def submit(self):
        logger.debug("Submit button clicked in RuleFormWindow.")
        if not self.on_submit:
            self.destroy()
            return
            
        updated_rule = {
            'id': self.rule.get('id') or f"rule-{int(datetime.datetime.now().timestamp())}",
            'name': self.name_var.get(),
            'priority': self.rule.get('priority', 99), # Keep priority internally for now
            'operation': self.operation_var.get(),
            'destination_pattern': self.dest_pattern_var.get(),
            'conditions': self.conditions
        }
        
        logger.debug(f"Calling on_submit with rule data: {updated_rule}")
        self.on_submit(updated_rule)
        self.destroy()

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
