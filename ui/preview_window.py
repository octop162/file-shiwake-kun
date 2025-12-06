import os
from typing import List, Dict, Any
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem, 
    QPushButton, QHBoxLayout, QHeaderView
)
from PySide6.QtCore import Qt

class PreviewWindow(QDialog):
    """
    A modal dialog to show the user what changes will be made and ask for confirmation.
    """
    def __init__(self, parent, results: List[Dict[str, Any]]):
        super().__init__(parent)
        self.setWindowTitle("プレビュー")
        self.resize(1000, 600)
        
        self.results = results
        self.result = "cancel"  # Default action
        
        self.create_widgets()
        self.populate_results()

    def create_widgets(self):
        layout = QVBoxLayout(self)

        summary_text = (f"以下のファイル操作が実行されます。よろしいですか？\n" 
                        f"合計 {len(self.results)}件の操作が予定されています。")
        
        summary_label = QLabel(summary_text)
        summary_label.setStyleSheet("font-size: 14px; font-weight: bold;")
        layout.addWidget(summary_label)
        
        # Tree Widget
        self.tree = QTreeWidget()
        self.tree.setHeaderLabels(['操作', '元のディレクトリ', '元のファイル名', '移動先ディレクトリ', '移動先ファイル名', '適用ルール'])
        
        # Configure columns
        header = self.tree.header()
        header.setSectionResizeMode(QHeaderView.Interactive) # Allow user resizing
        header.setStretchLastSection(False) # Disable stretching last section automatically
        
        # Set initial widths (optional but good for UX)
        header.resizeSection(0, 60)  # Operation
        header.resizeSection(1, 250) # Source Dir
        header.resizeSection(2, 150) # Source File
        header.resizeSection(3, 250) # Dest Dir
        header.resizeSection(4, 150) # Dest File
        header.resizeSection(5, 100) # Rule Name

        layout.addWidget(self.tree)
        
        # Buttons
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        cancel_btn = QPushButton("キャンセル")
        cancel_btn.clicked.connect(lambda: self.resolve("cancel"))
        button_layout.addWidget(cancel_btn)
        
        confirm_btn = QPushButton("実行")
        confirm_btn.setDefault(True)
        confirm_btn.clicked.connect(lambda: self.resolve("confirm"))
        button_layout.addWidget(confirm_btn)
        
        layout.addLayout(button_layout)

    def populate_results(self):
        """Populates the TreeWidget with the proposed operations."""
        for plan in self.results:
            rule = plan.get('rule')
            if not rule:
                continue

            source_path = plan.get('file_path')
            source_dir = os.path.dirname(source_path) if source_path else ''
            source_file = os.path.basename(source_path) if source_path else ''

            dest_path = plan.get('dest_path')
            dest_dir = os.path.dirname(dest_path) if dest_path else 'N/A'
            dest_file = os.path.basename(dest_path) if dest_path else 'N/A'

            item = QTreeWidgetItem([
                rule.get('operation', 'move'),
                source_dir,
                source_file,
                dest_dir,
                dest_file,
                rule.get('name', 'Unknown')
            ])
            item.setTextAlignment(0, Qt.AlignCenter)
            self.tree.addTopLevelItem(item)
            
    def resolve(self, choice: str):
        """Sets the result and closes the dialog."""
        self.result = choice
        self.accept() if choice == 'confirm' else self.reject()

if __name__ == '__main__':
    from PySide6.QtWidgets import QApplication
    import sys

    app = QApplication(sys.argv)
    
    test_results = [
        {
            'file_path': 'C:/Users/test/img1.jpg', 
            'dest_path': 'D:/Photos/2023/img1.jpg', 
            'rule': {'name': 'Sort Photos', 'operation': 'move'}
        },
        {
            'file_path': 'C:/Users/test/docs/report.pdf', 
            'dest_path': 'D:/Docs/Work/report.pdf', 
            'rule': {'name': 'Work Docs', 'operation': 'copy'}
        }
    ]
    
    dialog = PreviewWindow(None, test_results)
    if dialog.exec():
        print("Confirmed")
    else:
        print("Cancelled")
        
    sys.exit()
