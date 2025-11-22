import tkinter as tk
from tkinter import ttk, messagebox
from typing import Dict, Any, List
import uuid
import logging

from .rule_form_window import RuleFormWindow

logger = logging.getLogger(__name__)

class SettingsWindow(tk.Toplevel):
    """
    A Toplevel window for viewing and managing application rules.
    """
    def __init__(self, master, config: Dict[str, Any], on_save: callable):
        super().__init__(master)
        self.title("設定")
        self.geometry("900x700")
        
        logger.debug("Opening SettingsWindow.")
        
        self.transient(master)
        self.grab_set()
        
        self.config = config
        self.on_save = on_save
        # Make a deep copy to handle edits and cancellations correctly
        self.rules = [dict(r) for r in config.get('rules', [])]
        logger.debug(f"Initial rules loaded: {len(self.rules)} rules.")
        
        self.preview_mode_var = tk.BooleanVar(value=self.config.get('preview_mode', True))
        
        self.create_widgets()
        self.populate_rules()

    def create_widgets(self):
        """Creates the UI widgets for the settings window."""
        main_frame = ttk.Frame(self, padding="10")
        main_frame.pack(fill=tk.BOTH, expand=True)

        # --- General Settings ---
        general_frame = ttk.LabelFrame(main_frame, text="全般設定", padding="10")
        general_frame.pack(fill=tk.X, pady=5)
        
        preview_check = ttk.Checkbutton(
            general_frame,
            text="ファイル操作の前にプレビューを表示する",
            variable=self.preview_mode_var
        )
        preview_check.pack(side=tk.LEFT)

        # --- Rules List ---
        rules_frame = ttk.LabelFrame(main_frame, text="整理ルール (優先度順)", padding="10")
        rules_frame.pack(fill=tk.BOTH, expand=True, pady=5)
        
        self.tree = ttk.Treeview(
            rules_frame,
            columns=('name', 'value'),
            show='headings'
        )
        self.tree.heading('name', text='ルール / 条件')
        self.tree.heading('value', text='値')

        self.tree.column('name', width=400)
        self.tree.column('value', width=300)
        
        self.tree.pack(fill=tk.BOTH, expand=True)
        self.tree.bind("<Double-1>", self.edit_rule)
        
        # --- Rule Manipulation Buttons ---
        button_frame = ttk.Frame(main_frame)
        button_frame.pack(fill=tk.X, pady=10)
        
        ttk.Button(button_frame, text="新規ルール追加...", command=self.add_rule).pack(side=tk.LEFT)
        ttk.Button(button_frame, text="選択したルールを編集...", command=self.edit_rule).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text="選択したルールを削除", command=self.delete_rule).pack(side=tk.LEFT)
        
        # --- Action Buttons ---
        action_frame = ttk.Frame(main_frame)
        action_frame.pack(fill=tk.X, pady=(20, 0))
        
        ttk.Button(action_frame, text="保存して閉じる", command=self.save_and_close).pack(side=tk.RIGHT)
        ttk.Button(action_frame, text="キャンセル", command=self.destroy).pack(side=tk.RIGHT, padx=5)

    def populate_rules(self):
        """Populates the Treeview with the current rules in a hierarchical view."""
        logger.debug("Populating rules in Treeview.")
        for item in self.tree.get_children():
            self.tree.delete(item)
            
        sorted_rules = sorted(self.rules, key=lambda r: r.get('priority', 999))
        for i, rule in enumerate(sorted_rules):
            # Insert parent rule item
            rule_text = f"({rule.get('priority', 'N/A')}) {rule.get('name', '')} -> [{rule.get('operation', '')}]"
            parent_iid = self.tree.insert(
                '', tk.END, iid=f"rule_{i}", open=True,
                values=(rule_text, rule.get('destination_pattern', ''))
            )
            
            # Insert child condition items
            for j, cond in enumerate(rule.get('conditions', [])):
                condition_text = f"  └─ if {cond['field']} {cond['operator']}"
                self.tree.insert(
                    parent_iid, tk.END, iid=f"rule_{i}_cond_{j}",
                    values=(condition_text, str(cond['value']))
                )

    def add_rule(self):
        """Opens a form to add a new rule."""
        logger.debug("Add Rule button clicked.")
        def on_submit(new_rule):
            logger.debug(f"Submitting new rule: {new_rule}")
            new_rule['id'] = str(uuid.uuid4())
            self.rules.append(new_rule)
            logger.debug(f"Rules list now has {len(self.rules)} rules.")
            self.populate_rules()

        RuleFormWindow(self, on_submit=on_submit)

    def edit_rule(self, event=None):
        """Opens a form to edit the selected rule."""
        selected_item_iid = self.tree.focus()
        if not selected_item_iid:
            messagebox.showwarning("編集エラー", "編集するルールを選択してください。")
            return
        
        # We only want to edit parent rule items
        if not selected_item_iid.startswith("rule_") or "cond" in selected_item_iid:
             # Find parent if a child is selected
            parent_iid = self.tree.parent(selected_item_iid)
            if not parent_iid:
                return
            self.tree.focus(parent_iid) # set focus on parent
            selected_item_iid = parent_iid

        logger.debug(f"Edit Rule button clicked for item: {selected_item_iid}")
        rule_index = int(selected_item_iid.split('_')[1])
        original_rule = sorted(self.rules, key=lambda r: r.get('priority', 999))[rule_index]
        logger.debug(f"Editing rule: {original_rule}")

        def on_submit(edited_rule):
            logger.debug(f"Submitting edited rule: {edited_rule}")
            for i, r in enumerate(self.rules):
                if r['id'] == original_rule['id']:
                    self.rules[i] = edited_rule
                    break
            self.populate_rules()
        
        RuleFormWindow(self, rule=original_rule, on_submit=on_submit)

    def delete_rule(self):
        """Deletes the selected rule."""
        selected_item_iid = self.tree.focus()
        if not selected_item_iid:
            messagebox.showwarning("削除エラー", "削除するルールを選択してください。")
            return

        if not selected_item_iid.startswith("rule_") or "cond" in selected_item_iid:
            parent_iid = self.tree.parent(selected_item_iid)
            if not parent_iid: return
            self.tree.focus(parent_iid)
            selected_item_iid = parent_iid

        logger.debug(f"Delete Rule button clicked for item: {selected_item_iid}")
        rule_index = int(selected_item_iid.split('_')[1])
        rule_to_delete = sorted(self.rules, key=lambda r: r.get('priority', 999))[rule_index]
        
        if messagebox.askyesno("確認", f"本当にルール '{rule_to_delete['name']}' を削除しますか？"):
            logger.debug(f"Deleting rule: {rule_to_delete}")
            self.rules = [r for r in self.rules if r['id'] != rule_to_delete['id']]
            logger.debug(f"Rules list now has {len(self.rules)} rules.")
            self.populate_rules()

    def save_and_close(self):
        """Saves the configuration and closes the window."""
        logger.debug("Save & Close button clicked.")
        self.config['rules'] = self.rules
        self.config['preview_mode'] = self.preview_mode_var.get()
        self.on_save(self.config)
        self.destroy()

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
