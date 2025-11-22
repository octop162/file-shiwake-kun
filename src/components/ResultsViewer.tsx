/**
 * 処理結果表示コンポーネント
 * Results viewer component for displaying file processing results
 * 
 * 要件: 8.1, 8.2, 8.3, 8.5
 */

import { useState, useMemo } from 'react';
import { ProcessResult } from '../types';

interface ResultsViewerProps {
  results: ProcessResult[];
  onClear?: () => void;
}

/**
 * フィルタータイプ
 */
type FilterType = 'all' | 'success' | 'failure';

/**
 * 処理結果表示コンポーネント
 */
const ResultsViewer: React.FC<ResultsViewerProps> = ({ results, onClear }) => {
  const [filter, setFilter] = useState<FilterType>('all');
  const [expandedItems, setExpandedItems] = useState<Set<number>>(new Set());
  const [showLogViewer, setShowLogViewer] = useState(false);

  /**
   * フィルタリングされた結果
   */
  const filteredResults = useMemo(() => {
    switch (filter) {
      case 'success':
        return results.filter(r => r.success);
      case 'failure':
        return results.filter(r => !r.success);
      default:
        return results;
    }
  }, [results, filter]);

  /**
   * サマリー統計の計算
   * 要件 8.3: 処理完了時のサマリー表示
   */
  const summary = useMemo(() => {
    const successCount = results.filter(r => r.success).length;
    const failureCount = results.filter(r => !r.success).length;
    const total = results.length;
    const successRate = total > 0 ? ((successCount / total) * 100).toFixed(1) : '0.0';
    
    return {
      successCount,
      failureCount,
      total,
      successRate,
    };
  }, [results]);

  /**
   * アイテムの展開/折りたたみ
   */
  const toggleExpand = (index: number) => {
    setExpandedItems(prev => {
      const newSet = new Set(prev);
      if (newSet.has(index)) {
        newSet.delete(index);
      } else {
        newSet.add(index);
      }
      return newSet;
    });
  };

  /**
   * ログテキストの生成
   * 要件 8.5: トラブルシューティングのためのログ保持
   */
  const generateLogText = () => {
    const timestamp = new Date().toISOString();
    let log = `=== ファイル仕訳け君 処理ログ ===\n`;
    log += `生成日時: ${timestamp}\n`;
    log += `合計: ${summary.total} 件\n`;
    log += `成功: ${summary.successCount} 件\n`;
    log += `失敗: ${summary.failureCount} 件\n`;
    log += `成功率: ${summary.successRate}%\n`;
    log += `\n`;
    log += `=== 処理結果詳細 ===\n\n`;

    results.forEach((result, index) => {
      log += `[${index + 1}] ${result.success ? '✓ 成功' : '✗ 失敗'}\n`;
      log += `  元のパス: ${result.source_path}\n`;
      
      if (result.destination_path) {
        log += `  移動先: ${result.destination_path}\n`;
      }
      
      if (result.matched_rule) {
        log += `  適用ルール: ${result.matched_rule}\n`;
      }
      
      if (result.error_message) {
        log += `  エラー: ${result.error_message}\n`;
      }
      
      log += `\n`;
    });

    return log;
  };

  /**
   * ログのダウンロード
   */
  const downloadLog = () => {
    const logText = generateLogText();
    const blob = new Blob([logText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `file-shiwake-kun-log-${Date.now()}.txt`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  };

  /**
   * ログビューアーの表示/非表示
   */
  const toggleLogViewer = () => {
    setShowLogViewer(prev => !prev);
  };

  if (results.length === 0) {
    return (
      <div className="results-viewer empty">
        <p className="empty-message">処理結果はまだありません</p>
        <p className="empty-hint">ファイルをドラッグ&ドロップして処理を開始してください</p>
      </div>
    );
  }

  return (
    <div className="results-viewer">
      {/* サマリーセクション - 要件 8.3 */}
      <div className="results-summary">
        <h2>処理結果サマリー</h2>
        <div className="summary-cards">
          <div className="summary-card total">
            <div className="card-icon">📊</div>
            <div className="card-content">
              <div className="card-label">合計</div>
              <div className="card-value">{summary.total}</div>
            </div>
          </div>
          
          <div className="summary-card success">
            <div className="card-icon">✓</div>
            <div className="card-content">
              <div className="card-label">成功</div>
              <div className="card-value">{summary.successCount}</div>
            </div>
          </div>
          
          <div className="summary-card failure">
            <div className="card-icon">✗</div>
            <div className="card-content">
              <div className="card-label">失敗</div>
              <div className="card-value">{summary.failureCount}</div>
            </div>
          </div>
          
          <div className="summary-card rate">
            <div className="card-icon">%</div>
            <div className="card-content">
              <div className="card-label">成功率</div>
              <div className="card-value">{summary.successRate}%</div>
            </div>
          </div>
        </div>
      </div>

      {/* アクションバー */}
      <div className="results-actions">
        <div className="filter-buttons">
          <button
            className={`filter-btn ${filter === 'all' ? 'active' : ''}`}
            onClick={() => setFilter('all')}
          >
            すべて ({results.length})
          </button>
          <button
            className={`filter-btn ${filter === 'success' ? 'active' : ''}`}
            onClick={() => setFilter('success')}
          >
            成功 ({summary.successCount})
          </button>
          <button
            className={`filter-btn ${filter === 'failure' ? 'active' : ''}`}
            onClick={() => setFilter('failure')}
          >
            失敗 ({summary.failureCount})
          </button>
        </div>
        
        <div className="action-buttons">
          <button className="action-btn" onClick={toggleLogViewer}>
            {showLogViewer ? 'ログを閉じる' : 'ログを表示'}
          </button>
          <button className="action-btn" onClick={downloadLog}>
            ログをダウンロード
          </button>
          {onClear && (
            <button className="action-btn clear" onClick={onClear}>
              クリア
            </button>
          )}
        </div>
      </div>

      {/* ログビューアー - 要件 8.5 */}
      {showLogViewer && (
        <div className="log-viewer">
          <div className="log-viewer-header">
            <h3>処理ログ</h3>
            <button className="close-btn" onClick={toggleLogViewer}>×</button>
          </div>
          <pre className="log-content">{generateLogText()}</pre>
        </div>
      )}

      {/* 処理結果一覧 - 要件 8.1, 8.2 */}
      <div className="results-list">
        <h3>
          処理結果詳細
          {filter !== 'all' && (
            <span className="filter-label">
              ({filter === 'success' ? '成功のみ' : '失敗のみ'})
            </span>
          )}
        </h3>
        
        {filteredResults.length === 0 ? (
          <p className="no-results">
            {filter === 'success' ? '成功した処理はありません' : '失敗した処理はありません'}
          </p>
        ) : (
          <ul className="results-items">
            {filteredResults.map((result) => {
              const originalIndex = results.indexOf(result);
              const isExpanded = expandedItems.has(originalIndex);
              
              return (
                <li
                  key={originalIndex}
                  className={`result-item ${result.success ? 'success' : 'failure'} ${isExpanded ? 'expanded' : ''}`}
                >
                  <div className="result-header" onClick={() => toggleExpand(originalIndex)}>
                    <span className="result-status">
                      {result.success ? '✓' : '✗'}
                    </span>
                    <span className="result-filename">
                      {result.source_path.split(/[/\\]/).pop()}
                    </span>
                    <span className="expand-icon">
                      {isExpanded ? '▼' : '▶'}
                    </span>
                  </div>
                  
                  {isExpanded && (
                    <div className="result-details">
                      {/* 要件 8.1: 元のパスと移動先パスの表示 */}
                      <div className="detail-row">
                        <span className="detail-label">元のパス:</span>
                        <span className="detail-value path">{result.source_path}</span>
                      </div>
                      
                      {result.destination_path && (
                        <div className="detail-row">
                          <span className="detail-label">移動先:</span>
                          <span className="detail-value path">{result.destination_path}</span>
                        </div>
                      )}
                      
                      {result.matched_rule && (
                        <div className="detail-row">
                          <span className="detail-label">適用ルール:</span>
                          <span className="detail-value">{result.matched_rule}</span>
                        </div>
                      )}
                      
                      {/* 要件 8.2: エラーメッセージの表示 */}
                      {result.error_message && (
                        <div className="detail-row error">
                          <span className="detail-label">エラー:</span>
                          <span className="detail-value error-message">
                            {result.error_message}
                          </span>
                        </div>
                      )}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
};

export default ResultsViewer;
