# 設計書 (Python/Tkinter版)

## 概要

ファイル仕分君は、ファイルのメタデータ（EXIF情報、ファイルシステム属性）を読み取り、ユーザー定義のルールに基づいてファイルを自動的に整理する、PythonとTkinterで実装されるクロスプラットフォームGUIアプリケーションです。

### 技術スタック

- **言語**: Python 3.9+
- **GUI**: Tkinter (ttk, tkinter.dnd)
- **画像メタデータ**: Pillow
- **設定管理**: JSON
- **パッケージング**: PyInstaller

### アーキテクチャ

UIとビジネスロジックを分離した、単一プロセスのデスクトップアプリケーションとして構築します。

```
┌─────────────────────────────────────────┐
│              プレゼンテーション層 (UI)        │
│          (Tkinter, ttk, D&D)          │
└──────────────────┬────────────────────┘
                   │ direct function calls
┌──────────────────▼────────────────────┐
│            ビジネスロジック層 (Logic)     │
│ (ファイル処理オーケストレーター, ルールエンジン) │
└──────────────────┬────────────────────┘
                   │ direct function calls
┌──────────────────▼────────────────────┐
│            データアクセス層 (Data)      │
│ (メタデータ抽出, ファイル操作, 設定管理) │
└─────────────────────────────────────────┘
```

## コンポーネント構成

プロジェクトは以下のパッケージ/モジュールで構成されます。

- **`main.py`**: アプリケーションのメインエントリーポイント。
- **`ui/`**: GUI関連のすべてのモジュールを格納。
  - `main_window.py`: メインウィンドウ、D&Dエリアの管理。
  - `settings_window.py`: ルールを管理するためのトップレベルウィンドウ。
  - `results_view.py`: 処理結果を表示するためのフレーム。
  - `conflict_dialog.py`: ファイル名競合時に表示するダイアログ。
- **`logic/`**: アプリケーションのコアビジネスロジック。
  - `file_processor.py`: ファイル処理全体のオーケストレーション。
  - `rule_engine.py`: メタデータとルールのマッチング。
- **`data/`**: データ永続化とアクセスに関するモジュール。
  - `metadata_extractor.py`: `os`と`Pillow`を使ったメタデータ抽出。
  - `file_operations.py`: `os`と`shutil`を使ったファイル操作。
  - `config_manager.py`: `config.json`の読み書き。

## UIコンポーネント

### `MainWindow` (in `ui/main_window.py`)
メインウィンドウを管理し、D&D操作の受付と処理の開始を担います。

```python
class MainWindow(ttk.Frame):
    def __init__(self, master):
        super().__init__(master)
        # ...ウィジェットの初期化...
        # D&Dのセットアップ
        
    def handle_drop(self, files: list[str]):
        # file_processorを呼び出して処理を開始
        pass
```

### `SettingsWindow` (in `ui/settings_window.py`)
ルールの追加、編集、削除を行うUI。

```python
class SettingsWindow(tk.Toplevel):
    def __init__(self, master, config):
        super().__init__(master)
        # ...ウィジェットの初期化...

    def save_rules(self):
        # config_managerを使ってルールを保存
        pass
```

## ビジネスロジックコンポーネント

### `FileProcessor` (in `logic/file_processor.py`)
ファイル処理を統括します。UIからファイルリストを受け取り、各モジュールを連携させて処理を実行します。

```python
class FileProcessor:
    def __init__(self, config):
        self.rule_engine = RuleEngine(config['rules'])
        # ...

    def process_files(self, file_paths: list[str]) -> list[ProcessResult]:
        # メタデータ抽出、ルール適用、ファイル操作を順に実行
        pass
```

### `RuleEngine` (in `logic/rule_engine.py`)
メタデータとルールを照合し、適用すべきルールと移動先パスを決定します。

```python
class RuleEngine:
    def __init__(self, rules: list[dict]):
        self.rules = rules

    def find_matching_rule(self, metadata: dict) -> tuple[dict | None, str | None]:
        # メタデータにマッチするルールを探し、テンプレートを展開して移動先パスを返す
        pass
```

## データアクセスコンポーネント

### `MetadataExtractor` (in `data/metadata_extractor.py`)
ファイルパスからメタデータを抽出します。

```python
class MetadataExtractor:
    def extract(self, file_path: str) -> dict:
        # os.pathとPillow.Imageを使ってメタデータを抽出
        pass
```

### `FileOperations` (in `data/file_operations.py`)
実際のファイル移動・コピー処理を実行します。

```python
class FileOperations:
    def move_file(self, source: str, dest: str):
        # shutil.moveを使用
        pass
        
    def copy_file(self, source: str, dest: str):
        # shutil.copy2を使用
        pass
```

### `ConfigManager` (in `data/config_manager.py`)
設定ファイルを管理します。

```python
class ConfigManager:
    def __init__(self, config_path='config.json'):
        self.config_path = config_path

    def load_config(self) -> dict:
        # JSONファイルを読み込む
        pass

    def save_config(self, config: dict):
        # JSONファイルに保存
        pass
```

## データモデル

### 設定ファイル (`config.json`)
アプリケーションの設定とルールはJSON形式で保存します。

```json
{
  "default_destination": "C:/Unsorted",
  "preview_mode": false,
  "log_path": "file-shiwake-kun.log",
  "rules": [
    {
      "id": "rule-001",
      "name": "写真を年月別に整理",
      "priority": 1,
      "operation": "move",
      "destination_pattern": "D:/Photos/{year}/{month}",
      "conditions": [
        {
          "field": "extension",
          "operator": "in",
          "value": [".jpg", ".jpeg", ".png"]
        },
        {
          "field": "capture_date",
          "operator": "exists",
          "value": null
        }
      ]
    }
  ]
}
```

### テンプレート変数
移動先パスパターンで使用可能な変数は要件定義書に準じます。
- `{year}`, `{month}`, `{day}`, `{extension}`, `{camera}`, `{filename}` など。
