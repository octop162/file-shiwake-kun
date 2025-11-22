# インストールガイド

ファイル仕訳け君のインストール方法を説明します。

## システム要件

### Windows (要件 7.1)
- Windows 10 (64-bit) 以降
- 200 MB以上の空きディスク容量
- WebView2 Runtime（通常はWindows 10/11に含まれています）

### macOS (要件 7.2)
- macOS 10.15 (Catalina) 以降
- 200 MB以上の空きディスク容量
- Intel MacまたはApple Silicon (M1/M2/M3) Mac

### Linux (要件 7.3)
- GTK3をサポートするLinuxディストリビューション
- 200 MB以上の空きディスク容量
- WebKitGTK 4.1以降

## インストール手順

### Windows

#### 方法1: MSIインストーラー（推奨）

1. `ファイル仕訳け君_0.1.0_x64_en-US.msi` をダウンロード
2. ダウンロードしたファイルをダブルクリック
3. インストールウィザードの指示に従う
4. インストール完了後、スタートメニューから「ファイル仕訳け君」を起動

#### 方法2: NSISインストーラー

1. `ファイル仕訳け君_0.1.0_x64-setup.exe` をダウンロード
2. ダウンロードしたファイルをダブルクリック
3. インストールウィザードの指示に従う
4. インストール完了後、デスクトップまたはスタートメニューから起動

**注意**: Windows Defenderの警告が表示される場合があります。「詳細情報」→「実行」をクリックしてください。

### macOS

#### Intel Mac

1. `ファイル仕訳け君_0.1.0_x64.dmg` をダウンロード
2. ダウンロードしたDMGファイルをダブルクリック
3. 表示されたウィンドウで、アプリケーションアイコンをApplicationsフォルダにドラッグ
4. Applicationsフォルダから「ファイル仕訳け君」を起動

#### Apple Silicon Mac (M1/M2/M3)

1. `ファイル仕訳け君_0.1.0_aarch64.dmg` をダウンロード
2. ダウンロードしたDMGファイルをダブルクリック
3. 表示されたウィンドウで、アプリケーションアイコンをApplicationsフォルダにドラッグ
4. Applicationsフォルダから「ファイル仕訳け君」を起動

**初回起動時の注意**:

macOSのGatekeeperにより、初回起動時に警告が表示される場合があります：

1. 「"ファイル仕訳け君"は開発元を確認できないため開けません」と表示された場合
2. システム環境設定 → セキュリティとプライバシー を開く
3. 「このまま開く」ボタンをクリック

または、Controlキーを押しながらアプリケーションをクリックし、「開く」を選択してください。

### Linux

#### Debian/Ubuntu系（DEBパッケージ）

```bash
# ダウンロードしたDEBパッケージをインストール
sudo dpkg -i file-shiwake-kun_0.1.0_amd64.deb

# 依存関係の問題がある場合
sudo apt-get install -f

# アプリケーションの起動
file-shiwake-kun
```

#### AppImage（全ディストリビューション対応）

```bash
# ダウンロードしたAppImageに実行権限を付与
chmod +x file-shiwake-kun_0.1.0_amd64.AppImage

# アプリケーションの起動
./file-shiwake-kun_0.1.0_amd64.AppImage
```

**推奨**: AppImageをホームディレクトリまたは `/opt` に移動して使用してください。

#### Fedora/RHEL系

```bash
# DEBパッケージをRPMに変換（alien使用）
sudo dnf install alien
sudo alien -r file-shiwake-kun_0.1.0_amd64.deb
sudo dnf install file-shiwake-kun-0.1.0-1.x86_64.rpm
```

または、AppImageを使用してください。

#### Arch Linux

```bash
# AURからインストール（将来的に対応予定）
# 現在はAppImageを使用してください
```

## アンインストール

### Windows

1. 設定 → アプリ → インストールされているアプリ
2. 「ファイル仕訳け君」を検索
3. 「アンインストール」をクリック

または、コントロールパネルの「プログラムと機能」から削除できます。

### macOS

1. Applicationsフォルダを開く
2. 「ファイル仕訳け君.app」をゴミ箱にドラッグ
3. ゴミ箱を空にする

設定ファイルも削除する場合：
```bash
rm -rf ~/Library/Application\ Support/com.file-shiwake-kun.app
```

### Linux

#### DEBパッケージ

```bash
sudo apt remove file-shiwake-kun
```

#### AppImage

AppImageファイルを削除するだけです：
```bash
rm file-shiwake-kun_0.1.0_amd64.AppImage
```

設定ファイルも削除する場合：
```bash
rm -rf ~/.config/file-shiwake-kun
```

## トラブルシューティング

### Windows

**問題**: アプリケーションが起動しない

**解決策**:
1. WebView2 Runtimeがインストールされているか確認
2. [Microsoft WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) からダウンロードしてインストール

### macOS

**問題**: 「破損しているため開けません」と表示される

**解決策**:
```bash
xattr -cr /Applications/ファイル仕訳け君.app
```

### Linux

**問題**: AppImageが起動しない

**解決策**:
```bash
# FUSE がインストールされているか確認
sudo apt install fuse libfuse2  # Ubuntu/Debian
sudo dnf install fuse fuse-libs  # Fedora
```

**問題**: 依存関係エラー

**解決策**:
```bash
# 必要なライブラリをインストール
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1  # Ubuntu/Debian
sudo dnf install webkit2gtk4.1 libappindicator-gtk3  # Fedora
```

## サポート

問題が解決しない場合は、以下をご確認ください：

- [GitHub Issues](https://github.com/file-shiwake-kun/file-shiwake-kun/issues)
- [ドキュメント](https://github.com/file-shiwake-kun/file-shiwake-kun/wiki)

## 更新

新しいバージョンがリリースされた場合：

1. 現在のバージョンをアンインストール
2. 新しいバージョンをダウンロード
3. 上記のインストール手順に従ってインストール

設定ファイルは保持されるため、再設定の必要はありません。

---

**バージョン**: 0.1.0  
**最終更新**: 2024-11-22
