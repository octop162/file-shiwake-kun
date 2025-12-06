import os
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QLabel, QCheckBox, 
    QPushButton, QHBoxLayout, QWidget, QStyle
)
from PySide6.QtCore import Qt, Slot

class ConflictDialog(QDialog):
    """
    A modal dialog to resolve a file name conflict.
    """
    def __init__(self, parent, source_path: str, dest_path: str):
        super().__init__(parent)
        self.setWindowTitle("ファイル名の競合")
        self.resize(600, 250)
        
        # Center on parent if available
        if parent:
            self.setModal(True)
            
        self.source_path = source_path
        self.dest_path = dest_path
        self.result = {"resolution": "skip", "apply_to_all": False}  # Default action
        
        self.create_widgets()

    def create_widgets(self):
        layout = QVBoxLayout(self)
        
        # Icon and Message
        msg_layout = QHBoxLayout()
        icon_label = QLabel()
        icon_label.setPixmap(self.style().standardIcon(QStyle.SP_MessageBoxWarning).pixmap(32, 32))
        msg_layout.addWidget(icon_label, 0, Qt.AlignTop)
        
        message = (f"移動先に同名のファイルが既に存在します:\n" 
                   f"'{self.dest_path}'\n\n" 
                   f"どのように処理しますか？")
        msg_label = QLabel(message)
        msg_label.setWordWrap(True)
        msg_layout.addWidget(msg_label, 1)
        layout.addLayout(msg_layout)
        
        layout.addSpacing(10)

        # Checkbox
        self.apply_to_all_check = QCheckBox("以降の競合では常にこの選択を適用する")
        layout.addWidget(self.apply_to_all_check)
        
        layout.addSpacing(20)

        # Buttons
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        self.overwrite_btn = QPushButton("上書き")
        self.overwrite_btn.clicked.connect(lambda: self.resolve("overwrite"))
        button_layout.addWidget(self.overwrite_btn)
        
        self.skip_btn = QPushButton("スキップ")
        self.skip_btn.clicked.connect(lambda: self.resolve("skip"))
        button_layout.addWidget(self.skip_btn)
        
        self.rename_btn = QPushButton("名前を変更して保存")
        self.rename_btn.clicked.connect(lambda: self.resolve("rename"))
        button_layout.addWidget(self.rename_btn)
        
        layout.addLayout(button_layout)

    def resolve(self, choice: str):
        """Sets the result and closes the dialog."""
        resolution = choice
        if choice == "rename":
            base, ext = os.path.splitext(self.dest_path)
            # Simple rename logic for now, could be improved to find a free name
            resolution = f"{base}_1{ext}"
            
        self.result = {
            "resolution": resolution, 
            "apply_to_all": self.apply_to_all_check.isChecked()
        }
        self.accept()

if __name__ == '__main__':
    from PySide6.QtWidgets import QApplication
    import sys

    app = QApplication(sys.argv)
    
    dialog = ConflictDialog(None, "/src/test.txt", "/dest/test.txt")
    if dialog.exec():
        print(f"Result: {dialog.result}")
    
    sys.exit()

