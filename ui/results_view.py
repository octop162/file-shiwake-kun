import os
from typing import List, Dict, Any
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLabel, QTreeWidget, QTreeWidgetItem, 
    QPushButton, QHBoxLayout, QHeaderView
)
from PySide6.QtGui import QColor
from PySide6.QtCore import Qt

class ResultsView(QDialog):
    """
    A Toplevel window to display the results of a file processing operation.
    """
    def __init__(self, parent, results: List[Dict[str, Any]]):
        super().__init__(parent)
        self.setWindowTitle("処理結果")
        self.resize(1000, 600)
        
        self.results = results
        
        self.create_widgets()
        self.populate_results()

    def create_widgets(self):
        layout = QVBoxLayout(self)

        # --- Summary ---
        success_count = sum(1 for r in self.results if r.get('status') == 'success')
        skipped_count = sum(1 for r in self.results if r.get('status') == 'skipped')
        fail_count = sum(1 for r in self.results if r.get('status') == 'failed')
        summary_text = f"処理完了: 合計 {len(self.results)}件 (成功: {success_count}件, スキップ: {skipped_count}件, 失敗: {fail_count}件)"
        
        summary_label = QLabel(summary_text)
        summary_label.setStyleSheet("font-size: 14px; font-weight: bold;")
        layout.addWidget(summary_label)
        
        # --- Results Tree ---
        self.tree = QTreeWidget()
        self.tree.setHeaderLabels(['状態', '元のディレクトリ', '元のファイル名', '移動先ディレクトリ', '移動先ファイル名', '詳細'])
        
        header = self.tree.header()
        header.setSectionResizeMode(QHeaderView.ResizeToContents)
        header.setSectionResizeMode(1, QHeaderView.Stretch)
        header.setSectionResizeMode(3, QHeaderView.Stretch)
        
        layout.addWidget(self.tree)
        
        # --- Action Buttons ---
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        close_btn = QPushButton("閉じる")
        close_btn.clicked.connect(self.accept)
        button_layout.addWidget(close_btn)
        
        layout.addLayout(button_layout)

    def populate_results(self):
        """Populates the TreeWidget with the processing results."""
        for result in self.results:
            status = result.get('status', 'failed')
            status_text = {
                'success': "成功",
                'skipped': "スキップ",
                'failed': "失敗"
            }.get(status, "不明")

            # Details message
            details = result['error_message'] or result['matched_rule'] or "処理なし"
            if status == 'skipped' and not result.get('error_message'):
                details = "ルールに一致しないためスキップ"

            source_path = result['source_path']
            source_dir = os.path.dirname(source_path) if source_path else ''
            source_file = os.path.basename(source_path) if source_path else ''

            dest_path = result['destination_path']
            dest_dir = os.path.dirname(dest_path) if dest_path else 'N/A'
            dest_file = os.path.basename(dest_path) if dest_path else 'N/A'

            item = QTreeWidgetItem([
                status_text,
                source_dir,
                source_file,
                dest_dir,
                dest_file,
                str(details)
            ])
            
            item.setTextAlignment(0, Qt.AlignCenter)

            # Set colors based on status
            if status == 'success':
                item.setForeground(0, QColor('green'))
            elif status == 'failed':
                item.setForeground(0, QColor('red'))
            elif status == 'skipped':
                item.setForeground(0, QColor('orange'))
            
            self.tree.addTopLevelItem(item)

if __name__ == '__main__':
    from PySide6.QtWidgets import QApplication
    import sys

    app = QApplication(sys.argv)
    
    test_results = [
        {'source_path': 'C:/Users/test/img1.jpg', 'destination_path': 'D:/Photos/2023/img1.jpg', 'status': 'success', 'matched_rule': 'Sort Photos', 'error_message': None},
        {'source_path': 'C:/Users/test/doc1.pdf', 'destination_path': None, 'status': 'failed', 'matched_rule': None, 'error_message': 'Failed to copy file.'},
        {'source_path': 'C:/Users/test/notes.txt', 'destination_path': None, 'status': 'skipped', 'matched_rule': None, 'error_message': None},
    ]
    
    win = ResultsView(None, test_results)
    win.exec()
    
    sys.exit()
