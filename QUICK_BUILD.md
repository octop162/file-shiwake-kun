# クイックビルドガイド

このガイドでは、各プラットフォームでのビルド手順を簡潔に説明します。

## 前提条件の確認

```bash
# Node.js のバージョン確認
node --version  # 18以降が必要

# Rust のバージョン確認
rustc --version  # 1.91以降が必要

# npm のバージョン確認
npm --version
```

## 依存関係のインストール

```bash
# プロジェクトルートで実行
npm install
```

## 開発ビルド

```bash
# 開発モードで起動（ホットリロード有効）
npm run tauri:dev
```

## プロダクションビルド

### Windows

```powershell
# PowerShellで実行
.\scripts\build-all.ps1

# または
npm run tauri:build:windows
```

**生成物の場所**:
- `src-tauri\target\release\bundle\msi\` - MSIインストーラー
- `src-tauri\target\release\bundle\nsis\` - NSISインストーラー

### macOS

```bash
# Bashで実行
./scripts/build-all.sh

# または個別に
npm run tauri:build:macos      # Intel Mac
npm run tauri:build:macos-arm  # Apple Silicon
```

**生成物の場所**:
- `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/` - Intel版DMG
- `src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/` - Apple Silicon版DMG

### Linux

```bash
# Bashで実行
./scripts/build-all.sh

# または
npm run tauri:build:linux
```

**生成物の場所**:
- `src-tauri/target/release/bundle/deb/` - DEBパッケージ
- `src-tauri/target/release/bundle/appimage/` - AppImage

## ビルド前のチェック

```bash
# テストの実行
npm run test

# Rustテストの実行
cd src-tauri
cargo test --release
cd ..

# 型チェック
npx tsc --noEmit

# ビルドの確認（実際にはビルドしない）
npm run build
```

## トラブルシューティング

### ビルドが失敗する

```bash
# キャッシュをクリア
rm -rf node_modules
rm -rf src-tauri/target
npm install
```

### Rustの依存関係エラー

```bash
cd src-tauri
cargo clean
cargo update
cd ..
```

### フロントエンドのビルドエラー

```bash
rm -rf dist
npm run build
```

## ビルド時間の目安

- **開発ビルド（初回）**: 5-10分
- **開発ビルド（2回目以降）**: 30秒-2分
- **プロダクションビルド（初回）**: 10-20分
- **プロダクションビルド（2回目以降）**: 3-5分

## ビルドの最適化

### Rustのビルドキャッシュ

```bash
# sccacheを使用（オプション）
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### 並列ビルド

```bash
# Cargoの並列ジョブ数を設定
export CARGO_BUILD_JOBS=4
```

## CI/CDでのビルド

GitHub Actionsを使用した自動ビルド：

```bash
# タグをプッシュしてビルドをトリガー
git tag v0.1.0
git push origin v0.1.0
```

詳細は `.github/workflows/build.yml` を参照してください。

## 次のステップ

- 詳細なビルド手順: `BUILD.md`
- プラットフォーム固有の注意事項: `PLATFORM_NOTES.md`
- リリースチェックリスト: `RELEASE_CHECKLIST.md`
- インストールガイド: `INSTALL.md`

---

**最終更新日**: 2024-11-22
