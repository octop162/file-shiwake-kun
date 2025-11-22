# ビルドとパッケージングガイド

このドキュメントでは、ファイル仕訳け君をWindows、macOS、Linux向けにビルドおよびパッケージングする方法を説明します。

## 前提条件

### 共通要件

- Node.js 18以降
- Rust 1.91以降
- npm または yarn

### プラットフォーム固有の要件

#### Windows (要件 7.1)

- Windows 10以降
- Visual Studio 2019以降（C++ Build Tools）
- WebView2 Runtime（通常はWindows 10/11に含まれる）

**インストール手順:**

```powershell
# Rust のインストール
winget install Rustlang.Rustup

# Node.js のインストール
winget install OpenJS.NodeJS

# Visual Studio Build Tools のインストール
winget install Microsoft.VisualStudio.2022.BuildTools
```

#### macOS (要件 7.2)

- macOS 10.15 (Catalina) 以降
- Xcode Command Line Tools

**インストール手順:**

```bash
# Xcode Command Line Tools のインストール
xcode-select --install

# Homebrew のインストール（未インストールの場合）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Rust のインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js のインストール
brew install node
```

#### Linux (要件 7.3)

- GTK3をサポートするLinuxディストリビューション
- 必要なシステムライブラリ

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Rust のインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js のインストール
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

**Fedora/RHEL:**

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel

# Rust のインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js のインストール
sudo dnf install -y nodejs npm
```

**Arch Linux:**

```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg

# Rust のインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js のインストール
sudo pacman -S nodejs npm
```

## プロジェクトのセットアップ

```bash
# 依存関係のインストール
npm install

# Rust の依存関係の確認
cd src-tauri
cargo check
cd ..
```

## 開発ビルド

開発モードでアプリケーションを実行：

```bash
npm run tauri:dev
```

## プロダクションビルド

### Windows向けビルド

Windows上で実行：

```bash
# デフォルトターゲット（現在のアーキテクチャ）
npm run tauri:build

# または明示的に x86_64 をターゲット
npm run tauri:build:windows
```

**生成されるファイル:**

- `src-tauri/target/release/file-shiwake-kun.exe` - 実行ファイル
- `src-tauri/target/release/bundle/msi/ファイル仕訳け君_0.1.0_x64_en-US.msi` - MSIインストーラー
- `src-tauri/target/release/bundle/nsis/ファイル仕訳け君_0.1.0_x64-setup.exe` - NSISインストーラー

**インストーラーの種類:**

- **MSI**: Windows Installer形式。企業環境での展開に適している
- **NSIS**: Nullsoft Scriptable Install System。カスタマイズ可能なインストーラー

### macOS向けビルド

macOS上で実行：

```bash
# Intel Mac
npm run tauri:build:macos

# Apple Silicon (M1/M2/M3)
npm run tauri:build:macos-arm

# ユニバーサルバイナリ（両方のアーキテクチャ）
npm run tauri:build
```

**生成されるファイル:**

- `src-tauri/target/release/bundle/macos/ファイル仕訳け君.app` - アプリケーションバンドル
- `src-tauri/target/release/bundle/dmg/ファイル仕訳け君_0.1.0_x64.dmg` - DMGインストーラー

**コード署名（オプション）:**

macOSでアプリケーションを配布する場合、Apple Developer IDで署名することを推奨：

```bash
# 署名用の環境変数を設定
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_ID="your-apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"

# ビルド
npm run tauri:build
```

### Linux向けビルド

Linux上で実行：

```bash
# デフォルトターゲット
npm run tauri:build

# または明示的に x86_64 をターゲット
npm run tauri:build:linux
```

**生成されるファイル:**

- `src-tauri/target/release/file-shiwake-kun` - 実行ファイル
- `src-tauri/target/release/bundle/deb/file-shiwake-kun_0.1.0_amd64.deb` - Debianパッケージ
- `src-tauri/target/release/bundle/appimage/file-shiwake-kun_0.1.0_amd64.AppImage` - AppImage

**パッケージの種類:**

- **DEB**: Debian/Ubuntu系ディストリビューション用
- **AppImage**: ディストリビューション非依存の実行可能ファイル

## クロスプラットフォームビルド

### Windows上でLinux向けにビルド（WSL使用）

```bash
# WSL2でUbuntuをインストール
wsl --install -d Ubuntu

# WSL内で
cd /mnt/c/path/to/project
npm install
npm run tauri:build:linux
```

### GitHub Actionsを使用した自動ビルド

`.github/workflows/build.yml` を作成：

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [windows-latest, macos-latest, ubuntu-latest]
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt update
          sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
      
      - name: Install frontend dependencies
        run: npm install
      
      - name: Build application
        run: npm run tauri:build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}-build
          path: src-tauri/target/release/bundle/
```

## ビルドの最適化

### リリースビルドの最適化

`src-tauri/Cargo.toml` に以下を追加（既に設定済み）：

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### バンドルサイズの削減

1. **不要な依存関係の削除**
2. **アセットの最適化**（画像圧縮など）
3. **Tree-shakingの活用**（Viteが自動的に実行）

## トラブルシューティング

### Windows

**問題**: `error: linker 'link.exe' not found`

**解決策**: Visual Studio Build Toolsをインストール

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

### macOS

**問題**: `xcrun: error: invalid active developer path`

**解決策**: Xcode Command Line Toolsをインストール

```bash
xcode-select --install
```

### Linux

**問題**: `error: failed to run custom build command for 'openssl-sys'`

**解決策**: OpenSSL開発パッケージをインストール

```bash
# Ubuntu/Debian
sudo apt install libssl-dev

# Fedora
sudo dnf install openssl-devel
```

## 配布

### Windows

- MSIまたはNSISインストーラーを配布
- Microsoft Storeでの公開も可能

### macOS

- DMGファイルを配布
- Mac App Storeでの公開も可能（Apple Developer Program登録が必要）

### Linux

- DEBパッケージ: Debian/Ubuntu系ユーザー向け
- AppImage: 全ディストリビューション対応
- Flatpak/Snapパッケージの作成も検討可能

## 自動更新（将来の拡張）

Tauri 2.xは自動更新機能をサポートしています。実装する場合：

1. `tauri-plugin-updater` を追加
2. 更新サーバーの設定
3. 署名キーの生成と管理

詳細は [Tauri Updater Documentation](https://v2.tauri.app/plugin/updater/) を参照してください。

## 参考資料

- [Tauri Build Documentation](https://v2.tauri.app/distribute/)
- [Rust Cross Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Tauri Configuration](https://v2.tauri.app/reference/config/)
