/**
 * メインアプリケーションコンポーネント
 * Main application component with routing and state management
 */

import { useApp, useView, View } from './context';
import { MainWindow, SettingsPanel } from './components';
import { processFiles } from './api/tauri';
import './App.css';

function App() {
  const { currentView, goToMain, goToSettings } = useView();
  const { setProcessResults, setIsProcessing, error, clearError } = useApp();

  /**
   * ファイルドロップハンドラー
   */
  const handleFileDrop = async (files: string[]) => {
    setIsProcessing(true);
    clearError();
    
    try {
      const results = await processFiles(files);
      setProcessResults(results);
    } catch (err) {
      console.error('Failed to process files:', err);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="app">
      {/* ナビゲーションバー */}
      <nav className="navbar">
        <button 
          onClick={goToMain}
          className={currentView === View.Main ? 'active' : ''}
        >
          メイン
        </button>
        <button 
          onClick={goToSettings}
          className={currentView === View.Settings ? 'active' : ''}
        >
          設定
        </button>
      </nav>

      {/* エラー表示 */}
      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={clearError}>×</button>
        </div>
      )}

      {/* ビュー表示 */}
      <main className="main-content">
        {currentView === View.Main && (
          <MainWindow onFileDrop={handleFileDrop} />
        )}
        {currentView === View.Settings && (
          <SettingsPanel />
        )}
      </main>
    </div>
  );
}

export default App;
