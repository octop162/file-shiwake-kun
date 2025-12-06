import logging
import datetime
import os
from typing import Dict, Any, Optional

from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLabel, QLineEdit, QComboBox, 
    QPushButton, QHBoxLayout, QGridLayout, QGroupBox, QFileDialog, QMessageBox
)
from PySide6.QtCore import Qt

logger = logging.getLogger(__name__)

class RuleFormWindow(QDialog):
    """
    A Toplevel window for adding or editing a single rule.
    """
    def __init__(self, parent, rule: Optional[Dict[str, Any]] = None, on_submit: callable = None):
        super().__init__(parent)
        self.setWindowTitle("ルールの編集" if rule else "新規ルールの追加")
        self.resize(600, 550)
        
        # Center logic is handled by parent/layout management in Qt usually
        # but if we want to force center on parent:
        if parent:
            self.setModal(True)

        if rule:
            logger.debug(f"Opening RuleFormWindow to edit rule: {rule.get('id')}")
        else:
            logger.debug("Opening RuleFormWindow to add new rule.")

        self.rule = rule or {}
        self.on_submit = on_submit
        
        # Internal state
        self.conditions = self.rule.get('conditions', [])

        self.create_widgets()
        self.populate_fields()

    def create_widgets(self):
        layout = QVBoxLayout(self)

        # --- Basic Rule Info ---
        info_group = QGroupBox("基本情報")
        info_layout = QGridLayout()
        
        info_layout.addWidget(QLabel("ルール名:"), 0, 0)
        self.name_edit = QLineEdit()
        info_layout.addWidget(self.name_edit, 0, 1, 1, 2)

        info_layout.addWidget(QLabel("操作:"), 1, 0)
        # 固定でコピーのみを表示（以前はComboBoxでmoveも選べた）
        info_layout.addWidget(QLabel("コピー"), 1, 1, 1, 2)

        info_layout.addWidget(QLabel("移動先ディレクトリパターン:"), 2, 0)
        self.dest_path_edit = QLineEdit()
        info_layout.addWidget(self.dest_path_edit, 2, 1)
        
        browse_btn = QPushButton("参照...")
        browse_btn.clicked.connect(self._browse_directory)
        info_layout.addWidget(browse_btn, 2, 2)
        
        info_layout.addWidget(QLabel("移動先ファイル名パターン:"), 3, 0)
        self.dest_filename_edit = QLineEdit()
        info_layout.addWidget(self.dest_filename_edit, 3, 1, 1, 2)
        
        info_group.setLayout(info_layout)
        layout.addWidget(info_group)

        # --- Variables Hint ---
        hint_group = QGroupBox("利用可能な変数")
        hint_layout = QVBoxLayout()
        
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
        hint_label = QLabel(variables_hint_text)
        hint_label.setWordWrap(True)
        hint_layout.addWidget(hint_label)
        
        hint_group.setLayout(hint_layout)
        layout.addWidget(hint_group)

        # --- Action Buttons ---
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        cancel_btn = QPushButton("キャンセル")
        cancel_btn.clicked.connect(self.reject)
        button_layout.addWidget(cancel_btn)

        save_btn = QPushButton("保存")
        save_btn.clicked.connect(self.submit)
        save_btn.setDefault(True)
        button_layout.addWidget(save_btn)
        
        layout.addLayout(button_layout)

    def populate_fields(self):
        self.name_edit.setText(self.rule.get('name', ''))
        # self.op_combo.setCurrentText(self.rule.get('operation', 'move')) # Removed
        
        initial_dest_pattern = self.rule.get('destination_pattern', '').replace('\\', '/')
        dest_path_part = ""
        dest_filename_part = ""

        if '{filename}' in initial_dest_pattern or '{extension}' in initial_dest_pattern:
            last_slash_idx = initial_dest_pattern.rfind('/')
            if last_slash_idx != -1:
                dest_path_part = initial_dest_pattern[:last_slash_idx]
                dest_filename_part = initial_dest_pattern[last_slash_idx + 1:]
            else: 
                dest_filename_part = initial_dest_pattern
        else: 
            dest_path_part = initial_dest_pattern
            if not self.rule and not initial_dest_pattern:
                dest_filename_part = "{filename}.{extension}"
            
        self.dest_path_edit.setText(dest_path_part)
        self.dest_filename_edit.setText(dest_filename_part)

    def _browse_directory(self):
        """Opens a dialog to choose a directory."""
        directory = QFileDialog.getExistingDirectory(self, "移動先フォルダを選択")
        if directory:
            standardized_directory = directory.replace('\\', '/')
            self.dest_path_edit.setText(standardized_directory + '/')

    def submit(self):
        logger.debug("Submit button clicked in RuleFormWindow.")
            
        combined_dest_pattern = ""
        path_part = self.dest_path_edit.text().strip()
        filename_part = self.dest_filename_edit.text().strip()
        name_part = self.name_edit.text().strip()

        # Validation: Empty checks
        if not name_part:
            QMessageBox.warning(self, "入力エラー", "ルール名を入力してください。")
            self.name_edit.setFocus()
            return

        if not path_part:
            QMessageBox.warning(self, "入力エラー", "移動先ディレクトリパターンを入力してください。")
            self.dest_path_edit.setFocus()
            return

        # Validation: Directory existence check
        # If path contains variables (e.g. {year}), check the parent path before the first variable.
        check_path = path_part
        if '{' in check_path:
            # Cut off at the first variable occurrence
            idx = check_path.find('{')
            # Find the last slash before the variable to get a valid directory path
            last_slash = check_path.rfind('/', 0, idx)
            if last_slash == -1:
                last_slash = check_path.rfind('\\', 0, idx)
            
            if last_slash != -1:
                check_path = check_path[:last_slash]
            else:
                # If variable is at the start (e.g. "{year}/..."), we can't check much.
                # Maybe warn? For now, let's assume relative paths or root variables are risky but allowed if intended.
                # But user asked to check existence.
                # If it's purely a variable start, we might skip existence check or warn.
                # Let's check if the resulting check_path is not empty.
                if idx == 0:
                    check_path = "" 
        
        if check_path:
            # Normalize separators
            check_path = os.path.normpath(check_path)
            if not os.path.exists(check_path):
                reply = QMessageBox.warning(
                    self, 
                    "確認", 
                    f"指定されたフォルダ（または親フォルダ）が存在しません:\n{check_path}\n\nこのまま保存しますか？",
                    QMessageBox.Yes | QMessageBox.No,
                    QMessageBox.No
                )
                if reply == QMessageBox.No:
                    self.dest_path_edit.setFocus()
                    return

        if path_part:
            combined_dest_pattern = path_part
            if not combined_dest_pattern.endswith('/'):
                combined_dest_pattern += '/'
            combined_dest_pattern += filename_part
        else:
            combined_dest_pattern = filename_part

        updated_rule = {
            'id': self.rule.get('id') or f"rule-{int(datetime.datetime.now().timestamp())}",
            'name': name_part,
            'priority': self.rule.get('priority', 99),
            'operation': 'copy', # Always copy
            'destination_pattern': combined_dest_pattern,
            'conditions': self.conditions
        }
        
        logger.debug(f"Calling on_submit with rule data: {updated_rule}")
        if self.on_submit:
            self.on_submit(updated_rule)
        
        self.accept()

if __name__ == '__main__':
    from PySide6.QtWidgets import QApplication
    import sys
    import json

    app = QApplication(sys.argv)
    
    def show_result(rule):
        print("--- Rule Submitted ---")
        print(json.dumps(rule, indent=2))

    # Test Add
    # dialog = RuleFormWindow(None, on_submit=show_result)
    # dialog.exec()
    
    # Test Edit
    sample_rule = {
        "id": "rule-001", "name": "Sort Photos", "priority": 1,
        "operation": "move", "destination_pattern": "D:/Photos/{year}/{month}",
        "conditions": [{"field": "extension", "operator": "in", "value": [".jpg", ".jpeg"]}]
    }
    dialog = RuleFormWindow(None, rule=sample_rule, on_submit=show_result)
    dialog.exec()
    
    sys.exit()
