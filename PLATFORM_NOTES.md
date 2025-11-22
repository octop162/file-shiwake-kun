# プラットフォーム固有の注意事項

このドキュメントでは、各プラットフォームでのビルドと配布に関する重要な情報を記載します。

## Windows (要件 7.1)

### ビルド環境

- **対応バージョン**: Windows 10 (64-bit) 以降
- **必須コンポーネント**:
  - Visual Studio 2019以降のC++ Build Tools
  - WebView2 Runtime（Windows 10/11には通常含まれる）
  - Rust 1.91以降
  - Node.js 18以降

### インストーラー形式

#### MSI (Windows Installer)

**利点**:
- 企業環境での展開に適している
- グループポリシーでの配布が可能
- 標準的なWindowsインストーラー形式

**制限事項**:
- カスタマイズオプションが限定的
- ファイルサイズが大きい傾向

#### NSIS (Nullsoft Scriptable Install System)

**利点**:
- 高度なカスタマイズが可能
- 圧縮率が高い
- 多言語対応が容易

**制限事項**:
- 企業環境での展開には追加設定が必要

### パス処理

Windowsではバックスラッシュ (`\`) がパス区切り文字として使用されます。

```rust
// Rustでのクロスプラットフォーム対応
use std::path::PathBuf;

let path = PathBuf::from("C:\\Users\\username\\Documents");
// または
let path = PathBuf::from(r"C:\Users\username\Documents");
```

### ファイルシステムの特性

- **大文字小文字の区別**: なし（ただし保持される）
- **パスの最大長**: 260文字（レガシー）、32,767文字（長いパス対応時）
- **予約文字**: `< > : " / \ | ? *`
- **予約名**: `CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`

### コード署名

配布する場合は、コード署名証明書での署名を推奨：

```powershell
# SignToolを使用した署名
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com installer.exe
```

### WebView2の配布

アプリケーションはWebView2 Runtimeに依存します：

- **オプション1**: ユーザーに事前インストールを依頼
- **オプション2**: インストーラーにWebView2ブートストラッパーを含める
- **オプション3**: WebView2 Runtimeを固定バージョンでバンドル

## macOS (要件 7.2)

### ビルド環境

- **対応バージョン**: macOS 10.15 (Catalina) 以降
- **必須コンポーネント**:
  - Xcode Command Line Tools
  - Rust 1.91以降
  - Node.js 18以降

### アーキテクチャ

#### Intel (x86_64)

```bash
rustup target add x86_64-apple-darwin
npm run tauri:build:macos
```

#### Apple Silicon (aarch64)

```bash
rustup target add aarch64-apple-darwin
npm run tauri:build:macos-arm
```

#### ユニバーサルバイナリ

両方のアーキテクチャをサポートする単一のバイナリ：

```bash
# 両方のターゲットをビルド
npm run tauri:build:macos
npm run tauri:build:macos-arm

# lipoで結合
lipo -create \
  target/x86_64-apple-darwin/release/file-shiwake-kun \
  target/aarch64-apple-darwin/release/file-shiwake-kun \
  -output target/universal/file-shiwake-kun
```

### パス処理

macOSではスラッシュ (`/`) がパス区切り文字として使用されます。

```rust
let path = PathBuf::from("/Users/username/Documents");
```

### ファイルシステムの特性

- **大文字小文字の区別**: デフォルトではなし（APFS/HFS+）
- **パスの最大長**: 1024文字
- **予約文字**: `/` と NULL文字のみ
- **隠しファイル**: `.` で始まるファイル

### コード署名とノータリゼーション

App Storeや一般配布には署名とノータリゼーションが必要：

```bash
# 署名
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --options runtime \
  "ファイル仕訳け君.app"

# ノータリゼーション
xcrun notarytool submit "ファイル仕訳け君.dmg" \
  --apple-id "your-apple-id@example.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# ステープル
xcrun stapler staple "ファイル仕訳け君.app"
```

### Gatekeeperの対応

署名なしのアプリケーションは、初回起動時に警告が表示されます：

1. ユーザーはControlキーを押しながらクリック
2. 「開く」を選択
3. または、システム環境設定で許可

### DMGのカスタマイズ

`tauri.conf.json` でDMGの外観をカスタマイズ可能：

```json
{
  "bundle": {
    "macOS": {
      "dmg": {
        "background": "dmg-background.png",
        "windowSize": { "width": 600, "height": 400 },
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 420, "y": 170 }
      }
    }
  }
}
```

## Linux (要件 7.3)

### ビルド環境

- **対応ディストリビューション**: GTK3をサポートする全てのディストリビューション
- **必須コンポーネント**:
  - GTK3開発ライブラリ
  - WebKitGTK 4.1
  - Rust 1.91以降
  - Node.js 18以降

### パッケージ形式

#### DEB (Debian/Ubuntu)

```bash
# ビルド
npm run tauri:build:linux

# インストール
sudo dpkg -i target/release/bundle/deb/file-shiwake-kun_0.1.0_amd64.deb
```

**メタデータ**: `tauri.conf.json` の `bundle.linux.deb` セクションで設定

#### AppImage

ディストリビューション非依存の実行可能ファイル：

```bash
# 実行権限を付与
chmod +x file-shiwake-kun_0.1.0_amd64.AppImage

# 実行
./file-shiwake-kun_0.1.0_amd64.AppImage
```

**利点**:
- 依存関係を含む
- インストール不要
- 全ディストリビューションで動作

**制限事項**:
- ファイルサイズが大きい
- システム統合が限定的

### パス処理

Linuxではスラッシュ (`/`) がパス区切り文字として使用されます。

```rust
let path = PathBuf::from("/home/username/Documents");
```

### ファイルシステムの特性

- **大文字小文字の区別**: あり（ext4, btrfs等）
- **パスの最大長**: 4096文字（PATH_MAX）
- **予約文字**: `/` と NULL文字のみ
- **隠しファイル**: `.` で始まるファイル

### デスクトップ統合

`.desktop` ファイルでアプリケーションメニューに統合：

```desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=ファイル仕訳け君
Exec=file-shiwake-kun
Icon=file-shiwake-kun
Terminal=false
Categories=Utility;FileTools;
```

### 権限の処理

Linuxでは、ファイル操作に適切な権限が必要：

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

// ファイルの権限を設定
let mut perms = fs::metadata(path)?.permissions();
perms.set_mode(0o644);
fs::set_permissions(path, perms)?;
```

### ディストリビューション固有の注意事項

#### Ubuntu/Debian

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
```

#### Fedora/RHEL

```bash
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel
```

#### Arch Linux

```bash
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3
```

## クロスプラットフォーム開発のベストプラクティス

### パス処理

常に `std::path::PathBuf` を使用：

```rust
use std::path::PathBuf;

// 良い例
let path = PathBuf::from("relative/path");
let path = path.join("file.txt");

// 悪い例
let path = "relative/path/file.txt"; // プラットフォーム依存
```

### ファイル名の検証

プラットフォーム固有の制限を考慮：

```rust
fn is_valid_filename(name: &str) -> bool {
    // Windows予約文字
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    
    // Windows予約名
    let reserved_names = ["CON", "PRN", "AUX", "NUL"];
    
    !name.chars().any(|c| invalid_chars.contains(&c))
        && !reserved_names.contains(&name.to_uppercase().as_str())
}
```

### 改行コード

プラットフォームに応じた改行コードを使用：

```rust
#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";
```

### 環境変数

```rust
use std::env;

// ホームディレクトリの取得
let home = env::var("HOME")  // Unix
    .or_else(|_| env::var("USERPROFILE"))  // Windows
    .expect("Cannot determine home directory");
```

## テスト

各プラットフォームでのテストを推奨：

```bash
# 全プラットフォームでテスト実行
cargo test --release

# プラットフォーム固有のテスト
#[cfg(target_os = "windows")]
#[test]
fn test_windows_specific() {
    // Windowsのみのテスト
}
```

## 参考資料

- [Tauri Platform-Specific Configuration](https://v2.tauri.app/reference/config/)
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Cross-Platform File Paths in Rust](https://doc.rust-lang.org/std/path/)

---

**最終更新日**: 2024-11-22
