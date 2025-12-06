# ファイル仕分君

ファイル仕分君は、ファイルのメタデータ（画像のEXIF情報、ファイルシステム属性など）に基づいて、
ファイルを自動的に整理するデスクトップアプリケーションです。

## ローカルでのビルド

### 前提条件

-   Python 3.9以上

### セットアップ

```bash
pip install -r requirements.txt
```

### 実行可能ファイルのビルド

Nuitkaを使用して単一の実行可能ファイルをビルドします。

```bash
python -m nuitka --standalone --onefile --enable-plugin=pyside6 --windows-console-mode=disable --output-dir=dist --output-filename=FileShiwakeKun.exe --windows-icon-from-ico=icon.ico --assume-yes-for-downloads main.py
```

### 欲しい機能
- 進捗機能
- キャンセル
- スキップ上書き指定