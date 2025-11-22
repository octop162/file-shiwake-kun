import tkinter as tk
from tkinter import ttk, messagebox
from typing import Dict, Any, List
import uuid
import logging
from tkinterdnd2 import DND_FILES

from .rule_form_window import RuleFormWindow

logger = logging.getLogger(__name__)

class MainView(ttk.Frame):
    """
    The main view of the application, combining settings and D&D functionality.
    """
    def __init__(self, master, config: Dict[str, Any], on_save: callable, on_file_drop: callable):
        super().__init__(master, padding="10")
        
        logger.debug("Initializing MainView.")
        
        self.config = config
        self.on_save = on_save
        self.on_file_drop = on_file_drop
        # Make a deep copy to handle edits and cancellations correctly
        self.rules = [dict(r) for r in config.get('rules', [])]
        
        self.preview_mode_var = tk.BooleanVar(value=self.config.get('preview_mode', True))
        
        self.create_widgets()
        self.populate_rules()

    def create_widgets(self):
        """Creates the UI widgets for the main view."""
        
        # --- Drop Zone ---
        drop_zone_frame = ttk.LabelFrame(self, text="ここにファイルをドロップ", padding="10")
        drop_zone_frame.pack(fill=tk.X, pady=5)
        
        drop_label = ttk.Label(
            drop_zone_frame, 
            text="ファイルをドラッグ＆ドロップして処理を開始",
            font=('Helvetica', 12),
            anchor=tk.CENTER
        )
        drop_label.pack(pady=20, fill=tk.X, expand=True)
        
        # Register DND on the label and the frame
        drop_label.drop_target_register(DND_FILES)
        drop_label.dnd_bind('<<Drop>>', self.handle_drop)
        drop_zone_frame.drop_target_register(DND_FILES)
        drop_zone_frame.dnd_bind('<<Drop>>', self.handle_drop)

        # --- General Settings ---
        general_frame = ttk.LabelFrame(self, text="全般設定", padding="10")
        general_frame.pack(fill=tk.X, pady=10)
        
        preview_check = ttk.Checkbutton(
            general_frame,
            text="ファイル操作の前にプレビューを表示する",
            variable=self.preview_mode_var,
            command=self._save_settings
        )
        preview_check.pack(side=tk.LEFT)

        # --- Rules List ---
        rules_frame = ttk.LabelFrame(self, text="整理ルール", padding="10")
        rules_frame.pack(fill=tk.BOTH, expand=True, pady=5)
        
        self.tree = ttk.Treeview(rules_frame, columns=('name', 'value'), show='headings')
        self.tree.heading('name', text='ルール / 条件')
        self.tree.heading('value', text='値')
        self.tree.column('name', width=400)
        self.tree.column('value', width=300)
        self.tree.pack(fill=tk.BOTH, expand=True)
        self.tree.bind("<Double-1>", self.edit_rule)
        
        # --- Rule Manipulation Buttons and Exit Button ---
        button_frame = ttk.Frame(rules_frame)
        button_frame.pack(fill=tk.X, pady=(10,0))
        
        ttk.Button(button_frame, text="新規ルール追加...", command=self.add_rule).pack(side=tk.LEFT)
        ttk.Button(button_frame, text="選択したルールを編集...", command=self.edit_rule).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text="選択したルールを削除", command=self.delete_rule).pack(side=tk.LEFT)
        
        ttk.Button(button_frame, text="終了", command=self.master.destroy).pack(side=tk.RIGHT, padx=5)

    def handle_drop(self, event):
        """ Handles the file drop event. """
        logger.debug(f"Drop event data: {event.data}")
        path_string = event.data.strip()
        if path_string.startswith('{') and path_string.endswith('}'):
            path_string = path_string[1:-1]
        
        files = [p for p in path_string.split(' } {') if p]
        files = [p.replace('{', '').replace('}', '').strip() for p in files]

        if files and self.on_file_drop:
            self.on_file_drop(files)

    def populate_rules(self):
        # ... (same as before)
        logger.debug("Populating rules in Treeview.")
        for item in self.tree.get_children():
            self.tree.delete(item)
        sorted_rules = sorted(self.rules, key=lambda r: r.get('priority', 999))
        for i, rule in enumerate(sorted_rules):
            rule_text = f"({rule.get('priority', 'N/A')}) {rule.get('name', '')} -> [{rule.get('operation', '')}]"
            parent_iid = self.tree.insert(
                '', tk.END, iid=f"rule_{i}", open=True,
                values=(rule_text, rule.get('destination_pattern', ''))
            )
            for j, cond in enumerate(rule.get('conditions', [])):
                condition_text = f"  └─ if {cond['field']} {cond['operator']}"
                self.tree.insert(
                    parent_iid, tk.END, iid=f"rule_{i}_cond_{j}",
                    values=(condition_text, str(cond['value']))
                )

    def add_rule(self):
        # ... (same as before)
        logger.debug("Add Rule button clicked.")
        def on_submit(new_rule):
            new_rule['id'] = str(uuid.uuid4())
            self.rules.append(new_rule)
            self.populate_rules()
            self._save_settings()
        RuleFormWindow(self, on_submit=on_submit)

    def edit_rule(self, event=None):
        # ... (same as before)
        selected_item_iid = self.tree.focus()
        if not selected_item_iid: return
        if not selected_item_iid.startswith("rule_"):
            parent_iid = self.tree.parent(selected_item_iid)
            if not parent_iid: return
            self.tree.focus(parent_iid)
            selected_item_iid = parent_iid
        rule_index = int(selected_item_iid.split('_')[1])
        original_rule = sorted(self.rules, key=lambda r: r.get('priority', 999))[rule_index]
        def on_submit(edited_rule):
            for i, r in enumerate(self.rules):
                if r['id'] == original_rule['id']:
                    self.rules[i] = edited_rule
                    break
            self.populate_rules()
            self._save_settings()
        RuleFormWindow(self, rule=original_rule, on_submit=on_submit)

    def delete_rule(self):
        # ... (same as before)
        selected_item_iid = self.tree.focus()
        if not selected_item_iid: return
        if not selected_item_iid.startswith("rule_"):
            parent_iid = self.tree.parent(selected_item_iid)
            if not parent_iid: return
            self.tree.focus(parent_iid)
            selected_item_iid = parent_iid
        rule_index = int(selected_item_iid.split('_')[1])
        rule_to_delete = sorted(self.rules, key=lambda r: r.get('priority', 999))[rule_index]
        if messagebox.askyesno("確認", f"本当にルール '{rule_to_delete['name']}' を削除しますか？"):
            self.rules = [r for r in self.rules if r['id'] != rule_to_delete['id']]
            self.populate_rules()
            self._save_settings()

    def _save_settings(self):
        """Saves the current state of the settings."""
        logger.debug("Auto-saving settings...")
        self.config['rules'] = self.rules
        self.config['preview_mode'] = self.preview_mode_var.get()
        self.on_save(self.config)
        # Optional: Show a temporary "saved" message
        # For now, logging is enough, and a message box on every change would be annoying.
        # messagebox.showinfo("保存完了", "設定を保存しました。")


if __name__ == '__main__':
    # A simple test to run this window standalone
    import uuid

    class TestApp(tk.Tk):
        def __init__(self):
            super().__init__()
            self.title("Settings Window Test")
            
            self.test_config = {
                "rules": [
                    {
                        "id": str(uuid.uuid4()), "name": "Sort Photos by Year/Month", "priority": 1,
                        "operation": "move", "destination_pattern": "D:/Photos/{year}/{month}", "conditions": []
                    },
                    {
                        "id": str(uuid.uuid4()), "name": "Sort Documents", "priority": 2,
                        "operation": "copy", "destination_pattern": "C:/Documents/PDFs", "conditions": []
                    }
                ]
            }
            
            def on_save(config):
                print("--- Config Saved ---")
                import json
                print(json.dumps(config, indent=2))
                self.test_config = config

            ttk.Button(self, text="Open Settings", command=lambda: SettingsWindow(self, self.test_config, on_save)).pack(pady=50)

    app = TestApp()
    app.mainloop()
