/**
 * メインウィンドウコンポーネント
 * Main window component with drag & drop functionality
 */

import { useState } from 'react';
import { useApp } from '../context';

interface MainWindowProps {
  onFileDrop: (files: string[]) => void;
}

const MainWindow: React.FC<MainWindowProps> = ({ onFileDrop }) => {
  const { config, isProcessing, processResults } = useApp();
  const [isDragActive, setIsDragActive] = useState(false);
  const [dragCounter, setDragCounter] = useState(0);

  /**
   * ドラッグ開始時の処理
   * 視覚的フィードバックを有効化
   */
  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    setDragCounter(prev => prev + 1);
    
    if (e.dataTransfer.items && e.dataTransfer.items.length > 0) {
      setIsDragActive(true);
    }
  };

  /**
   * ドラッグ中の処理
   * ドロップを許可
   */
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  /**
   * ドラッグ終了時の処理
   * 視覚的フィードバックを無効化
   */
  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    setDragCounter(prev => prev - 1);
    
    if (dragCounter - 1 === 0) {
      setIsDragActive(false);
    }
  };

  /**
   * ドロップ時の処理
   * ファイルパスを抽出して処理を開始
   */
  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    setIsDragActive(false);
    setDragCounter(0);
    
    // ファイルパスを抽出
    const files: string[] = [];
    
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      // Tauri環境では、e.dataTransfer.filesからファイルパスを取得
      for (let i = 0; i < e.dataTransfer.files.length; i++) {
        const file = e.dataTransfer.files[i];
        // Tauriでは、file.pathプロパティにファイルパスが含まれる
        const filePath = (file as any).path || file.name;
        files.push(filePath);
      }
      
      if (files.length > 0) {
        onFileDrop(files);
      }
    }
  };

  /**
   * 処理結果のサマリーを計算
   */
  const getResultsSummary = () => {
    const successCount = processResults.filter(r => r.success).length;
    const failureCount = processResults.filter(r => !r.success).length;
    return { successCount, failureCount, total: processResults.length };
  };

  const summary = processResults.length > 0 ? getResultsSummary() : null;

  return (
    <div className="main-window">
      <h1>ファイル仕訳け君</h1>
      <p className="subtitle">File Shiwake-kun - Automatic File Organizer</p>
      
      <div 
        className={`drop-zone ${isDragActive ? 'drag-active' : ''} ${isProcessing ? 'processing' : ''}`}
        onDragEnter={handleDragEnter}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {isProcessing ? (
          <>
            <div className="spinner"></div>
            <p className="processing-text">処理中...</p>
            <p className="hint">Processing files...</p>
          </>
        ) : isDragActive ? (
          <>
            <p className="drop-text">ここにドロップ！</p>
            <p className="hint">Drop files here!</p>
          </>
        ) : (
          <>
            <div className="drop-icon">📁</div>
            <p>ファイルをここにドラッグ&ドロップ</p>
            <p className="hint">Drag and drop files or folders here</p>
            <p className="hint-detail">複数のファイルやフォルダを同時に処理できます</p>
          </>
        )}
      </div>

      {/* 進捗表示 */}
      {isProcessing && (
        <div className="progress-info">
          <div className="progress-bar">
            <div className="progress-bar-fill"></div>
          </div>
          <p className="progress-text">ファイルを処理しています...</p>
        </div>
      )}

      {/* 処理結果サマリー */}
      {summary && (
        <div className="results-summary">
          <h2>処理結果サマリー</h2>
          <div className="summary-stats">
            <div className="stat success">
              <span className="stat-icon">✓</span>
              <span className="stat-label">成功</span>
              <span className="stat-value">{summary.successCount}</span>
            </div>
            <div className="stat failure">
              <span className="stat-icon">✗</span>
              <span className="stat-label">失敗</span>
              <span className="stat-value">{summary.failureCount}</span>
            </div>
            <div className="stat total">
              <span className="stat-icon">Σ</span>
              <span className="stat-label">合計</span>
              <span className="stat-value">{summary.total}</span>
            </div>
          </div>
        </div>
      )}

      {/* 処理結果詳細 */}
      {processResults.length > 0 && (
        <div className="results">
          <h3>処理結果詳細</h3>
          <ul>
            {processResults.map((result, index) => (
              <li key={index} className={result.success ? 'success' : 'failure'}>
                <span className="result-icon">{result.success ? '✓' : '✗'}</span>
                <div className="result-content">
                  <div className="result-path">
                    <strong>元:</strong> {result.source_path}
                  </div>
                  {result.destination_path && (
                    <div className="result-path">
                      <strong>先:</strong> {result.destination_path}
                    </div>
                  )}
                  {result.matched_rule && (
                    <div className="result-rule">
                      <strong>ルール:</strong> {result.matched_rule}
                    </div>
                  )}
                  {result.error_message && (
                    <div className="result-error">
                      <strong>エラー:</strong> {result.error_message}
                    </div>
                  )}
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 設定情報 */}
      {config && (
        <div className="config-info">
          <h3>現在の設定</h3>
          <div className="config-details">
            <p>
              <span className="config-label">ルール数:</span>
              <span className="config-value">{config.rules.length}</span>
            </p>
            <p>
              <span className="config-label">プレビューモード:</span>
              <span className={`config-value ${config.preview_mode ? 'enabled' : 'disabled'}`}>
                {config.preview_mode ? 'ON' : 'OFF'}
              </span>
            </p>
            <p>
              <span className="config-label">デフォルト移動先:</span>
              <span className="config-value">{config.default_destination}</span>
            </p>
          </div>
        </div>
      )}
    </div>
  );
};

export default MainWindow;
