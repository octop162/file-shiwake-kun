/**
 * メインアプリケーションコンポーネント
 * Main application component with routing and state management
 */

import { useState, useEffect } from 'react';
import { useApp, useView, View } from './context';
import { MainWindow, SettingsPanel, ResultsViewer, PreviewDialog, ToastContainer } from './components';
import { processFiles } from './api/tauri';
import { ProcessResult } from './types';
import { useToast } from './hooks/useToast';
import { logger } from './utils/logger';

function App() {
  const { currentView, goToMain, goToSettings, goToResults } = useView();
  const { config, setProcessResults, setIsProcessing, processResults, error, clearError } = useApp();
  const [showPreview, setShowPreview] = useState(false);
  const [previewResults, setPreviewResults] = useState<ProcessResult[]>([]);
  const [pendingFiles, setPendingFiles] = useState<string[]>([]);
  const [isDarkMode, setIsDarkMode] = useState(() => {
    return document.documentElement.classList.contains('dark');
  });
  const toast = useToast();

  /**
   * ダークモード切り替え
   */
  const toggleDarkMode = () => {
    setIsDarkMode(prev => {
      const newMode = !prev;
      if (newMode) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      return newMode;
    });
  };

  // システムテーマ変更の監視
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      setIsDarkMode(e.matches);
    };
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, []);

  /**
   * ファイルドロップハンドラー
   */
  const handleFileDrop = async (files: string[]) => {
    logger.info('File drop initiated', { fileCount: files.length });
    setIsProcessing(true);
    clearError();
    
    try {
      const results = await processFiles(files);
      logger.info('File processing completed', { 
        totalFiles: results.length,
        successCount: results.filter(r => r.success).length,
        failureCount: results.filter(r => !r.success).length
      });
      
      // プレビューモードが有効な場合
      if (config?.preview_mode) {
        setPreviewResults(results);
        setPendingFiles(files);
        setShowPreview(true);
        setIsProcessing(false);
        toast.info('プレビュー結果を確認してください');
        logger.info('Preview mode: showing preview results');
      } else {
        // プレビューモードが無効な場合は直接実行
        setProcessResults(results);
        goToResults();
        setIsProcessing(false);
        
        const successCount = results.filter(r => r.success).length;
        const failureCount = results.filter(r => !r.success).length;
        
        if (failureCount === 0) {
          toast.success(`${successCount}件のファイルを正常に処理しました`);
        } else if (successCount === 0) {
          toast.error(`${failureCount}件のファイルの処理に失敗しました`);
        } else {
          toast.warning(`${successCount}件成功、${failureCount}件失敗`);
        }
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'ファイル処理に失敗しました';
      logger.error('File processing failed', { error: errorMessage, files });
      toast.error(errorMessage);
      setIsProcessing(false);
    }
  };

  /**
   * プレビュー確認ハンドラー
   * プレビュー結果を確認して実際の処理を実行
   */
  const handlePreviewConfirm = async () => {
    logger.info('Preview confirmed, executing file operations');
    setShowPreview(false);
    setIsProcessing(true);
    clearError();
    
    try {
      // 一時的にプレビューモードを無効化して実行
      if (config) {
        const tempConfig = { ...config, preview_mode: false };
        await import('./api/tauri').then(async ({ saveConfig }) => {
          await saveConfig(tempConfig);
        });
      }
      
      // 実際のファイル処理を実行
      const results = await processFiles(pendingFiles);
      setProcessResults(results);
      
      const successCount = results.filter(r => r.success).length;
      const failureCount = results.filter(r => !r.success).length;
      
      logger.info('File operations executed', { successCount, failureCount });
      
      if (failureCount === 0) {
        toast.success(`${successCount}件のファイルを正常に処理しました`);
      } else if (successCount === 0) {
        toast.error(`${failureCount}件のファイルの処理に失敗しました`);
      } else {
        toast.warning(`${successCount}件成功、${failureCount}件失敗`);
      }
      
      // プレビューモードを元に戻す
      if (config) {
        await import('./api/tauri').then(async ({ saveConfig }) => {
          await saveConfig(config);
        });
      }
      
      goToResults();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'ファイル処理の実行に失敗しました';
      logger.error('Failed to execute file processing', { error: errorMessage });
      toast.error(errorMessage);
    } finally {
      setIsProcessing(false);
      setPendingFiles([]);
      setPreviewResults([]);
    }
  };

  /**
   * プレビューキャンセルハンドラー
   * プレビュー結果を破棄
   */
  const handlePreviewCancel = () => {
    logger.info('Preview cancelled');
    toast.info('プレビューをキャンセルしました');
    setShowPreview(false);
    setPreviewResults([]);
    setPendingFiles([]);
  };

  /**
   * 処理結果をクリア
   */
  const handleClearResults = () => {
    setProcessResults([]);
    goToMain();
  };

  return (
    <div className="flex flex-col min-h-screen bg-light-bg dark:bg-dark-bg transition-colors duration-200">
      {/* Skip to main content for accessibility */}
      <a href="#main-content" className="skip-to-main">
        メインコンテンツへスキップ
      </a>

      {/* ナビゲーションバー */}
      <nav 
        className="flex items-center gap-4 px-4 py-3 bg-light-surface dark:bg-dark-surface border-b border-light-border dark:border-dark-border shadow-sm"
        role="navigation"
        aria-label="メインナビゲーション"
      >
        <button 
          onClick={goToMain}
          className={`btn btn-small transition-all ${
            currentView === View.Main 
              ? 'btn-primary' 
              : 'bg-white dark:bg-dark-elevated hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-gray-100'
          }`}
          aria-current={currentView === View.Main ? 'page' : undefined}
        >
          メイン
        </button>
        <button 
          onClick={goToSettings}
          className={`btn btn-small transition-all ${
            currentView === View.Settings 
              ? 'btn-primary' 
              : 'bg-white dark:bg-dark-elevated hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-gray-100'
          }`}
          aria-current={currentView === View.Settings ? 'page' : undefined}
        >
          設定
        </button>
        <button 
          onClick={goToResults}
          className={`btn btn-small transition-all ${
            currentView === View.Results 
              ? 'btn-primary' 
              : 'bg-white dark:bg-dark-elevated hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-gray-100'
          }`}
          disabled={processResults.length === 0}
          aria-current={currentView === View.Results ? 'page' : undefined}
          aria-label={`処理結果 ${processResults.length > 0 ? `${processResults.length}件` : ''}`}
        >
          処理結果 {processResults.length > 0 && `(${processResults.length})`}
        </button>

        {/* ダークモード切り替え */}
        <button
          onClick={toggleDarkMode}
          className="ml-auto btn btn-small bg-white dark:bg-dark-elevated hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-gray-100"
          aria-label={isDarkMode ? 'ライトモードに切り替え' : 'ダークモードに切り替え'}
          title={isDarkMode ? 'ライトモードに切り替え' : 'ダークモードに切り替え'}
        >
          {isDarkMode ? '☀️' : '🌙'}
        </button>
      </nav>

      {/* エラー表示 */}
      {error && (
        <div 
          className="flex justify-between items-center px-4 py-3 bg-error text-white shadow-md"
          role="alert"
          aria-live="assertive"
        >
          <span>{error}</span>
          <button 
            onClick={clearError}
            className="text-white hover:text-gray-200 text-2xl px-2 transition-colors"
            aria-label="エラーを閉じる"
          >
            ×
          </button>
        </div>
      )}

      {/* ビュー表示 */}
      <main 
        id="main-content"
        className="flex-1 p-4 md:p-8 overflow-y-auto scrollbar-thin"
        role="main"
      >
        {currentView === View.Main && (
          <MainWindow onFileDrop={handleFileDrop} />
        )}
        {currentView === View.Settings && (
          <SettingsPanel />
        )}
        {currentView === View.Results && (
          <ResultsViewer results={processResults} onClear={handleClearResults} />
        )}
      </main>

      {/* プレビューダイアログ */}
      {showPreview && (
        <PreviewDialog
          previewResults={previewResults}
          onConfirm={handlePreviewConfirm}
          onCancel={handlePreviewCancel}
        />
      )}

      {/* トースト通知 */}
      <ToastContainer toasts={toast.toasts} onClose={toast.removeToast} />
    </div>
  );
}

export default App;
