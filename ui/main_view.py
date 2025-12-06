import logging
import os
from typing import Dict, Any, List
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem, 
    QPushButton, QProgressBar, QGroupBox, QCheckBox, QMenu, QHeaderView, QMessageBox
)
from PySide6.QtCore import Qt, Signal, QSize
from PySide6.QtGui import QAction, QDragEnterEvent, QDropEvent

from .rule_form_window import RuleFormWindow

logger = logging.getLogger(__name__)

class DropZone(QGroupBox):
    """
    A specific area for dropping files.
    """
    fileDropped = Signal(list)

    def __init__(self):
        super().__init__("ここにファイルをドロップ")
        self.setAcceptDrops(True)
        self.setMinimumHeight(400) # Set minimum height for the drop zone
        layout = QVBoxLayout(self)
        
        self.label = QLabel("処理に使うルールを選択してから、\nファイルをドラッグ＆ドロップしてください")
        self.label.setAlignment(Qt.AlignCenter)
        self.label.setStyleSheet("font-size: 14px; color: #555;")
        layout.addWidget(self.label)
        
        # Progress Overlay (initially hidden)
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 0) # Indeterminate
        self.progress_bar.hide()
        layout.addWidget(self.progress_bar)
        
        self.progress_text = QLabel("")
        self.progress_text.setAlignment(Qt.AlignCenter)
        self.progress_text.hide()
        layout.addWidget(self.progress_text)

        # Cancel Button (initially hidden)
        self.cancel_btn = QPushButton("キャンセル")
        self.cancel_btn.hide()
        layout.addWidget(self.cancel_btn)

    def dragEnterEvent(self, event: QDragEnterEvent):
        if event.mimeData().hasUrls():
            logger.debug("Drag enter detected with URLs.")
            event.acceptProposedAction()
        else:
            logger.debug("Drag enter detected but NO URLs found.")

    def dropEvent(self, event: QDropEvent):
        try:
            logger.debug("Drop event detected.")
            urls = event.mimeData().urls()
            file_paths = []
            error_messages = []
            
            for url in urls:
                local_file = url.toLocalFile()
                if local_file:
                    if os.path.exists(local_file):
                        file_paths.append(local_file)
                    else:
                        error_messages.append(f"ファイルが見つかりません: {local_file}")
                else:
                    error_messages.append(f"URLをローカルパスに変換できませんでした: {url.toString()}")
            
            logger.debug(f"Processed file paths: {file_paths}")
            
            if error_messages:
                logger.warning(f"Drop errors: {error_messages}")
                error_text = "\n".join(error_messages[:5]) # Limit display
                if len(error_messages) > 5:
                    error_text += f"\n...他 {len(error_messages) - 5} 件"
                QMessageBox.warning(self, "読み込み警告", f"一部のファイルを読み込めませんでした:\n{error_text}")
            
            if file_paths:
                event.acceptProposedAction()
                self.fileDropped.emit(file_paths)
            else:
                logger.warning("No valid file paths found in drop event.")
                if not error_messages:
                    QMessageBox.warning(self, "読み込みエラー", "ドロップされたデータから有効なファイルパスを取得できませんでした。")

        except Exception as e:
            logger.exception("Error during drop event")
            QMessageBox.critical(self, "エラー", f"ドラッグ＆ドロップ処理中にエラーが発生しました:\n{str(e)}")
    
    def set_processing(self, processing: bool, text: str = ""):
        if processing:
            self.label.hide()
            self.progress_bar.show()
            self.progress_text.setText(text)
            self.progress_text.show()
            self.cancel_btn.show()
        else:
            self.label.show()
            self.progress_bar.hide()
            self.progress_text.hide()
            self.cancel_btn.hide()
            
    def update_progress_text(self, text: str):
        self.progress_text.setText(text)

class MainView(QWidget):
    """
    The main view of the application.
    """
    cancelRequested = Signal()

    def __init__(self, parent, config: Dict[str, Any], on_save: callable, on_file_drop: callable):
        super().__init__(parent)
        
        logger.debug("Initializing MainView.")
        
        self.config = config
        self.on_save = on_save
        self.on_file_drop = on_file_drop
        
        # Keep a local copy of rules to manipulate before saving
        self.rules = [dict(r) for r in config.get('rules', [])]
        
        self.create_widgets()
        self.populate_rules()

    def create_widgets(self):
        layout = QVBoxLayout(self)
        
        # --- Drop Zone ---
        self.drop_zone = DropZone()
        self.drop_zone.fileDropped.connect(self.handle_drop)
        self.drop_zone.cancel_btn.clicked.connect(self.cancelRequested)
        layout.addWidget(self.drop_zone)

        # --- General Settings ---
        # Preview checkbox removed as it's always on now
        # settings_group = QGroupBox("全般設定")
        # settings_layout = QHBoxLayout()
        #
        # self.preview_check = QCheckBox("ファイル操作の前にプレビューを表示する")
        # self.preview_check.setChecked(self.config.get('preview_mode', True))
        # self.preview_check.toggled.connect(self._save_settings)
        # settings_layout.addWidget(self.preview_check)
        #
        # settings_group.setLayout(settings_layout)
        # layout.addWidget(settings_group) # Removed
        
        # --- Rules List ---
        rules_group = QGroupBox("整理ルール")
        rules_layout = QVBoxLayout()
        
        self.tree = QTreeWidget()
        self.tree.setHeaderLabels(['ルール名', '操作', '移動先パターン'])
        self.tree.header().setSectionResizeMode(0, QHeaderView.Stretch)
        self.tree.header().setSectionResizeMode(2, QHeaderView.Stretch)
        self.tree.setContextMenuPolicy(Qt.CustomContextMenu)
        self.tree.customContextMenuRequested.connect(self._show_context_menu)
        self.tree.itemDoubleClicked.connect(self.edit_rule)
        
        rules_layout.addWidget(self.tree)
        
        # Buttons
        btn_layout = QHBoxLayout()
        
        add_btn = QPushButton("新規ルール追加...")
        add_btn.clicked.connect(self.add_rule)
        btn_layout.addWidget(add_btn)
        
        edit_btn = QPushButton("選択したルールを編集...")
        edit_btn.clicked.connect(self.edit_rule)
        btn_layout.addWidget(edit_btn)
        
        del_btn = QPushButton("選択したルールを削除")
        del_btn.clicked.connect(self.delete_rule)
        btn_layout.addWidget(del_btn)
        
        btn_layout.addStretch()
        rules_layout.addLayout(btn_layout)
        
        rules_group.setLayout(rules_layout)
        layout.addWidget(rules_group)

    def populate_rules(self):
        self.tree.clear()
        for rule in self.rules:
            item = QTreeWidgetItem([
                rule.get('name', 'No Name'),
                rule.get('operation', 'move'),
                rule.get('destination_pattern', '')
            ])
            item.setData(0, Qt.UserRole, rule['id']) # Store ID
            self.tree.addTopLevelItem(item)

    def handle_drop(self, file_paths: List[str]):
        self.on_file_drop(file_paths)

    def show_processing_state(self, is_processing: bool):
        self.drop_zone.set_processing(is_processing)
        self.tree.setEnabled(not is_processing)

    def update_progress_text(self, text: str):
        self.drop_zone.update_progress_text(text)

    def get_selected_rule_id(self) -> str:
        item = self.tree.currentItem()
        if item:
            return item.data(0, Qt.UserRole)
        return None

    def select_rule_by_id(self, rule_id: str):
        root = self.tree.invisibleRootItem()
        for i in range(root.childCount()):
            item = root.child(i)
            if item.data(0, Qt.UserRole) == rule_id:
                self.tree.setCurrentItem(item)
                break

    def add_rule(self):
        dialog = RuleFormWindow(self, on_submit=self._on_rule_added)
        dialog.exec()

    def _on_rule_added(self, new_rule):
        self.rules.append(new_rule)
        self.populate_rules()
        self._save_settings()

    def edit_rule(self):
        selected_id = self.get_selected_rule_id()
        if not selected_id:
            return
        
        rule = next((r for r in self.rules if r['id'] == selected_id), None)
        if rule:
            dialog = RuleFormWindow(self, rule=rule, on_submit=self._on_rule_updated)
            dialog.exec()

    def _on_rule_updated(self, updated_rule):
        for i, r in enumerate(self.rules):
            if r['id'] == updated_rule['id']:
                self.rules[i] = updated_rule
                break
        self.populate_rules()
        self.select_rule_by_id(updated_rule['id'])
        self._save_settings()

    def open_rule_folder(self):
        selected_id = self.get_selected_rule_id()
        if not selected_id:
            return
        
        rule = next((r for r in self.rules if r['id'] == selected_id), None)
        if not rule:
            return

        dest_pattern = rule.get('destination_pattern', '').replace('\\', '/')
        if not dest_pattern:
            return

        # Filename/Extension pattern separation logic similar to RuleFormWindow
        folder_path = dest_pattern
        if '{filename}' in dest_pattern or '{extension}' in dest_pattern:
            last_slash_idx = dest_pattern.rfind('/')
            if last_slash_idx != -1:
                folder_path = dest_pattern[:last_slash_idx]
            else:
                folder_path = ""

        # Handle variables: truncate at first variable
        if '{' in folder_path:
            idx = folder_path.find('{')
            folder_path = folder_path[:idx]
        
        folder_path = os.path.normpath(folder_path)

        # Find the deepest existing parent directory
        while folder_path and not os.path.exists(folder_path):
            parent = os.path.dirname(folder_path)
            if parent == folder_path:
                break
            folder_path = parent
            
        if folder_path and os.path.exists(folder_path):
            try:
                os.startfile(folder_path)
            except Exception as e:
                QMessageBox.warning(self, "エラー", f"フォルダを開けませんでした:\n{e}")
        else:
            QMessageBox.warning(self, "エラー", f"有効なフォルダパスが見つかりません:\n{dest_pattern}")

    def delete_rule(self):
        selected_id = self.get_selected_rule_id()
        if not selected_id:
            return

        reply = QMessageBox.question(
            self, "確認", "本当にこのルールを削除しますか？",
            QMessageBox.Yes | QMessageBox.No, QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            self.rules = [r for r in self.rules if r['id'] != selected_id]
            self.populate_rules()
            self._save_settings()

    def _save_settings(self):
        self.config['rules'] = self.rules
        # self.config['preview_mode'] = self.preview_check.isChecked() # Preview check always on now
        self.on_save(self.config)

    def _show_context_menu(self, position):
        menu = QMenu()
        edit_action = QAction("編集", self)
        edit_action.triggered.connect(self.edit_rule)
        menu.addAction(edit_action)
        
        delete_action = QAction("削除", self)
        delete_action.triggered.connect(self.delete_rule)
        menu.addAction(delete_action)

        open_folder_action = QAction("設定しているフォルダを開く", self)
        open_folder_action.triggered.connect(self.open_rule_folder)
        menu.addAction(open_folder_action)
        
        menu.exec(self.tree.viewport().mapToGlobal(position))