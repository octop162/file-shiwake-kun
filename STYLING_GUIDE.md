# スタイリングガイド

## Tailwind CSSの使用方法

### 基本的なユーティリティクラス

#### レイアウト
```tsx
// Flexbox
<div className="flex items-center justify-between gap-4">

// Grid
<div className="grid grid-cols-3 gap-4">

// Spacing
<div className="p-4 m-2">  // padding: 1rem, margin: 0.5rem
<div className="px-4 py-2"> // padding-x: 1rem, padding-y: 0.5rem
```

#### カラー
```tsx
// 背景色
<div className="bg-white dark:bg-dark-bg">

// テキスト色
<span className="text-gray-900 dark:text-gray-100">

// ボーダー色
<div className="border border-light-border dark:border-dark-border">
```

#### タイポグラフィ
```tsx
<h1 className="text-2xl font-bold">
<p className="text-sm text-gray-600">
```

### カスタムコンポーネントクラス

#### ボタン
```tsx
// プライマリボタン
<button className="btn btn-primary">保存</button>

// セカンダリボタン
<button className="btn btn-secondary">キャンセル</button>

// 危険なアクション
<button className="btn btn-danger">削除</button>

// 成功アクション
<button className="btn btn-success">確認</button>

// 小さいボタン
<button className="btn btn-small">編集</button>
```

#### カード
```tsx
<div className="card p-4">
  <h3>カードタイトル</h3>
  <p>カードコンテンツ</p>
</div>
```

#### 入力フィールド
```tsx
<label className="label">ラベル</label>
<input type="text" className="input" />

<select className="select">
  <option>オプション1</option>
</select>
```

#### ダイアログ
```tsx
<div className="dialog-overlay">
  <div className="dialog">
    <div className="p-6">
      <h2>ダイアログタイトル</h2>
      <p>ダイアログコンテンツ</p>
    </div>
  </div>
</div>
```

### レスポンシブデザイン

#### ブレークポイント
```tsx
// モバイル: デフォルト
// タブレット: md: (768px以上)
// デスクトップ: lg: (1024px以上)

<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
  {/* モバイル: 1列, タブレット: 2列, デスクトップ: 3列 */}
</div>

<div className="text-sm md:text-base lg:text-lg">
  {/* レスポンシブなフォントサイズ */}
</div>
```

### ダークモード

#### 自動切り替え
```tsx
// ライトモードとダークモードで異なるスタイル
<div className="bg-white dark:bg-dark-surface">
<p className="text-gray-900 dark:text-gray-100">
```

#### カスタムカラー
```javascript
// tailwind.config.js で定義済み
colors: {
  primary: '#646cff',
  success: '#4caf50',
  warning: '#ff9800',
  error: '#ff4444',
  dark: {
    bg: '#242424',
    surface: '#1a1a1a',
    elevated: '#2a2a2a',
    border: '#333',
  },
  light: {
    bg: '#ffffff',
    surface: '#f5f5f5',
    elevated: '#ffffff',
    border: '#e0e0e0',
  }
}
```

### アクセシビリティ

#### ARIA属性
```tsx
// ナビゲーション
<nav role="navigation" aria-label="メインナビゲーション">
  <button aria-current="page">現在のページ</button>
</nav>

// アラート
<div role="alert" aria-live="assertive">
  エラーメッセージ
</div>

// ボタンラベル
<button aria-label="メニューを開く" title="メニューを開く">
  ☰
</button>
```

#### フォーカス管理
```tsx
// フォーカス可能な要素
<button className="focus:outline-none focus:ring-2 focus:ring-primary">

// スキップリンク
<a href="#main-content" className="skip-to-main">
  メインコンテンツへスキップ
</a>
```

### アニメーション

#### トランジション
```tsx
// 基本的なトランジション
<div className="transition-all duration-200">

// ホバー効果
<button className="hover:bg-gray-100 transition-colors">
```

#### カスタムアニメーション
```tsx
// フロートアニメーション
<div className="animate-float">📁</div>

// フェードイン
<div className="animate-fadeIn">
```

#### アニメーション削減
```css
/* prefers-reduced-motion が有効な場合、アニメーションを削減 */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

### ユーティリティクラス

#### スクロールバー
```tsx
<div className="overflow-y-auto scrollbar-thin">
  {/* カスタムスクロールバー */}
</div>
```

#### ステータスインジケーター
```tsx
<span className="status-success">✓ 成功</span>
<span className="status-error">✗ エラー</span>
<span className="status-warning">⚠ 警告</span>
```

## ベストプラクティス

### 1. 一貫性のあるスペーシング
```tsx
// 良い例: 一貫したスペーシング
<div className="p-4 gap-4">
  <div className="mb-4">...</div>
  <div className="mb-4">...</div>
</div>

// 避けるべき: 不規則なスペーシング
<div className="p-3 gap-5">
  <div className="mb-2">...</div>
  <div className="mb-6">...</div>
</div>
```

### 2. セマンティックなHTML
```tsx
// 良い例
<nav>
  <button>ナビゲーション</button>
</nav>

// 避けるべき
<div>
  <div onClick={...}>ナビゲーション</div>
</div>
```

### 3. レスポンシブファースト
```tsx
// 良い例: モバイルファースト
<div className="text-sm md:text-base lg:text-lg">

// 避けるべき: デスクトップファースト
<div className="text-lg md:text-base sm:text-sm">
```

### 4. ダークモード対応
```tsx
// 良い例: 両方のモードで適切な色
<div className="bg-white dark:bg-dark-surface text-gray-900 dark:text-gray-100">

// 避けるべき: ダークモードで読めない
<div className="bg-white text-gray-900">
```

### 5. アクセシビリティ
```tsx
// 良い例: 適切なARIA属性
<button aria-label="閉じる" onClick={onClose}>×</button>

// 避けるべき: ラベルなし
<button onClick={onClose}>×</button>
```

## トラブルシューティング

### ダークモードが機能しない
```typescript
// main.tsx で初期化を確認
if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
  document.documentElement.classList.add('dark');
}
```

### スタイルが適用されない
1. `src/index.css` が `main.tsx` でインポートされているか確認
2. Tailwind設定の `content` パスが正しいか確認
3. ビルドを再実行: `npm run build`

### レスポンシブが機能しない
```html
<!-- index.html でビューポート設定を確認 -->
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
```

## 参考リンク

- [Tailwind CSS公式ドキュメント](https://tailwindcss.com/docs)
- [WCAG 2.1ガイドライン](https://www.w3.org/WAI/WCAG21/quickref/)
- [MDN Web Docs - アクセシビリティ](https://developer.mozilla.org/ja/docs/Web/Accessibility)
