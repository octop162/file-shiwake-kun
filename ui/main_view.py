import tkinter as tk
from tkinter import ttk, messagebox
from typing import Dict, Any, List
import uuid
import logging
import re
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
        self.rules = [dict(r) for r in config.get('rules', [])]
        
        self.preview_mode_var = tk.BooleanVar(value=self.config.get('preview_mode', True))
        
        self.create_widgets()
        self.populate_rules()

    def create_widgets(self):
        """Creates the UI widgets for the main view."""
        
        # --- Drop Zone ---
        self.drop_zone_frame = ttk.LabelFrame(self, text="ここにファイルをドロップ", padding="10")
        self.drop_zone_frame.pack(fill=tk.X, pady=5)
        
        self.drop_label = ttk.Label(
            self.drop_zone_frame, 
            text="処理に使うルールを選択してから、ファイルをドラッグ＆ドロップしてください",
            font=('Helvetica', 12),
            anchor=tk.CENTER
        )
        self.drop_label.pack(pady=20, fill=tk.X, expand=True)
        
        # --- Processing State Overlay for Drop Zone ---
        self.processing_overlay = ttk.Frame(self.drop_zone_frame)
        self.spinner = ttk.Progressbar(self.processing_overlay, mode='indeterminate')
        self.spinner.pack(pady=10)
        ttk.Label(self.processing_overlay, text="処理中...").pack(pady=5)
        
        self.drop_label.drop_target_register(DND_FILES)
        self.drop_label.dnd_bind('<<Drop>>', self.handle_drop)
        self.drop_zone_frame.drop_target_register(DND_FILES)
        self.drop_zone_frame.dnd_bind('<<Drop>>', self.handle_drop)

        # --- General Settings ---
        general_frame = ttk.LabelFrame(self, text="全般設定", padding="10")
        general_frame.pack(fill=tk.X, pady=10)
        
        self.preview_check = ttk.Checkbutton(
            general_frame, text="ファイル操作の前にプレビューを表示する",
            variable=self.preview_mode_var, command=self._save_settings
        )
        self.preview_check.pack(side=tk.LEFT)

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
        
        self.add_button = ttk.Button(button_frame, text="新規ルール追加...", command=self.add_rule)
        self.add_button.pack(side=tk.LEFT)
        self.edit_button = ttk.Button(button_frame, text="選択したルールを編集...", command=self.edit_rule)
        self.edit_button.pack(side=tk.LEFT, padx=5)
        self.delete_button = ttk.Button(button_frame, text="選択したルールを削除", command=self.delete_rule)
        self.delete_button.pack(side=tk.LEFT)
        
        self.exit_button = ttk.Button(button_frame, text="終了", command=self.master.destroy)
        self.exit_button.pack(side=tk.RIGHT, padx=5)

    def get_selected_rule_id(self) -> str | None:
        """Returns the ID of the currently selected rule in the tree."""
        selected_iid = self.tree.focus()
        if not selected_iid: return None
        if "cond" in selected_iid:
            return self.tree.parent(selected_iid)
        return selected_iid

    def show_processing_state(self, is_processing: bool):
        """Displays or hides the processing overlay."""
        if is_processing:
            self.drop_label.pack_forget()
            self.processing_overlay.place(relx=0.5, rely=0.5, anchor=tk.CENTER)
            self.spinner.start()
        else:
            self.spinner.stop()
            self.processing_overlay.place_forget()
            self.drop_label.pack(pady=20, fill=tk.X, expand=True)

    def select_rule_by_id(self, rule_id: str):
        """Sets the selection and focus on a specific rule in the treeview."""
        if rule_id and self.tree.exists(rule_id):
            logger.debug(f"Programmatically selecting rule: {rule_id}")
            self.tree.selection_set(rule_id)
            self.tree.focus(rule_id)
        
    def handle_drop(self, event):
        """ Handles the file drop event with a more robust parser. """
        logger.debug(f"Drop event data: {event.data}")
        # Use regex to find either braced content or non-space sequences
        path_string = event.data.strip()
        files = re.findall(r'({[^{}]*}|\S+)', path_string)
        # Clean up braces and any extra whitespace
        cleaned_files = [f.strip('{}').strip() for f in files]
        
        logger.debug(f"Parsed files: {cleaned_files}")

        if cleaned_files and self.on_file_drop:
            self.on_file_drop(cleaned_files)

    def populate_rules(self):
        # ... (same as before, no priority)
        logger.debug("Populating rules in Treeview.")
        for item in self.tree.get_children():
            self.tree.delete(item)
        for rule in self.rules:
            rule_id = rule.get('id')
            if not rule_id: continue
            rule_text = f"{rule.get('name', '')} -> [{rule.get('operation', '')}]"
            parent_iid = self.tree.insert(
                '', tk.END, iid=rule_id, open=True,
                values=(rule_text, rule.get('destination_pattern', ''))
            )
            for j, cond in enumerate(rule.get('conditions', [])):
                condition_text = f"  └─ if {cond['field']} {cond['operator']}"
                self.tree.insert(
                    parent_iid, tk.END, iid=f"{rule_id}_cond_{j}",
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
        # ... (same as before, uses rule id)
        selected_iid = self.get_selected_rule_id()
        if not selected_iid:
            messagebox.showwarning("編集エラー", "編集するルールを選択してください。")
            return
        original_rule = next((r for r in self.rules if r['id'] == selected_iid), None)
        if not original_rule: return
        def on_submit(edited_rule):
            for i, r in enumerate(self.rules):
                if r['id'] == original_rule['id']:
                    self.rules[i] = edited_rule
                    break
            self.populate_rules()
            self._save_settings()
        RuleFormWindow(self, rule=original_rule, on_submit=on_submit)

    def delete_rule(self):
        # ... (same as before, uses rule id)
        selected_iid = self.get_selected_rule_id()
        if not selected_iid:
            messagebox.showwarning("削除エラー", "削除するルールを選択してください。")
            return
        rule_to_delete = next((r for r in self.rules if r['id'] == selected_iid), None)
        if not rule_to_delete: return
        if messagebox.askyesno("確認", f"本当にルール '{rule_to_delete['name']}' を削除しますか？"):
            self.rules = [r for r in self.rules if r['id'] != rule_to_delete['id']]
            self.populate_rules()
            self._save_settings()

    def _save_settings(self):
        # ... (same as before)
        logger.debug("Auto-saving settings...")
        self.config['rules'] = self.rules
        self.config['preview_mode'] = self.preview_mode_var.get()
        self.on_save(self.config)


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
