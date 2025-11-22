/**
 * 競合解決ダイアログコンポーネント
 * Conflict resolution dialog component
 * 
 * 要件 11.1, 11.2, 11.3, 11.4, 11.5 を実装
 */

import { useState } from 'react';
import { FileInfo, ConflictResolution } from '../types';

interface ConflictDialogProps {
  sourceFile: FileInfo;
  destFile: FileInfo;
  onResolve: (resolution: ConflictResolution, applyToAll: boolean) => void;
  onCancel?: () => void;
}

/**
 * ファイルサイズを人間が読みやすい形式に変換
 */
const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

/**
 * タイムスタンプを日時文字列に変換
 */
const formatDateTime = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleString('ja-JP', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
};

const ConflictDialog: React.FC<ConflictDialogProps> = ({
  sourceFile,
  destFile,
  onResolve,
  onCancel,
}) => {
  const [applyToAll, setApplyToAll] = useState(false);

  /**
   * 解決オプションを処理
   */
  const handleResolve = (resolution: ConflictResolution) => {
    onResolve(resolution, applyToAll);
  };

  return (
    <div className="dialog-overlay">
      <div className="dialog conflict-dialog">
        <div className="dialog-header">
          <h2>⚠️ ファイル競合</h2>
          <p className="dialog-subtitle">File Conflict Detected</p>
        </div>

        <div className="dialog-content">
          <p className="conflict-message">
            移動先に同名のファイルが既に存在します。どのように処理しますか？
          </p>

          {/* ファイル情報比較 */}
          <div className="file-comparison">
            {/* 移動元ファイル */}
            <div className="file-info source-file">
              <h3>📄 移動元ファイル</h3>
              <div className="file-details">
                <div className="file-detail-row">
                  <span className="detail-label">ファイル名:</span>
                  <span className="detail-value">{sourceFile.name}</span>
                </div>
                <div className="file-detail-row">
                  <span className="detail-label">サイズ:</span>
                  <span className="detail-value">{formatFileSize(sourceFile.size)}</span>
                </div>
                <div className="file-detail-row">
                  <span className="detail-label">更新日時:</span>
                  <span className="detail-value">{formatDateTime(sourceFile.mod_time)}</span>
                </div>
              </div>
            </div>

            {/* 矢印 */}
            <div className="comparison-arrow">→</div>

            {/* 移動先ファイル */}
            <div className="file-info dest-file">
              <h3>📁 移動先ファイル</h3>
              <div className="file-details">
                <div className="file-detail-row">
                  <span className="detail-label">ファイル名:</span>
                  <span className="detail-value">{destFile.name}</span>
                </div>
                <div className="file-detail-row">
                  <span className="detail-label">サイズ:</span>
                  <span className="detail-value">{formatFileSize(destFile.size)}</span>
                </div>
                <div className="file-detail-row">
                  <span className="detail-label">更新日時:</span>
                  <span className="detail-value">{formatDateTime(destFile.mod_time)}</span>
                </div>
              </div>
            </div>
          </div>

          {/* 「以降も同様に処理」チェックボックス */}
          <div className="apply-to-all-option">
            <label>
              <input
                type="checkbox"
                checked={applyToAll}
                onChange={(e) => setApplyToAll(e.target.checked)}
              />
              <span>以降も同様に処理する（Apply to all conflicts）</span>
            </label>
          </div>
        </div>

        <div className="dialog-actions">
          {/* 個別解決オプション */}
          <div className="action-group individual-actions">
            <button
              className="btn btn-danger"
              onClick={() => handleResolve(ConflictResolution.Overwrite)}
              title="既存のファイルを上書きします"
            >
              上書き (Overwrite)
            </button>
            <button
              className="btn btn-secondary"
              onClick={() => handleResolve(ConflictResolution.Skip)}
              title="このファイルをスキップします"
            >
              スキップ (Skip)
            </button>
            <button
              className="btn btn-primary"
              onClick={() => handleResolve(ConflictResolution.Rename)}
              title="ファイル名を変更して保存します"
            >
              リネーム (Rename)
            </button>
          </div>

          {/* バッチ解決オプション */}
          <div className="action-group batch-actions">
            <button
              className="btn btn-danger btn-small"
              onClick={() => handleResolve(ConflictResolution.OverwriteAll)}
              title="すべての競合で上書きします"
            >
              すべて上書き (Overwrite All)
            </button>
            <button
              className="btn btn-secondary btn-small"
              onClick={() => handleResolve(ConflictResolution.SkipAll)}
              title="すべての競合をスキップします"
            >
              すべてスキップ (Skip All)
            </button>
            <button
              className="btn btn-primary btn-small"
              onClick={() => handleResolve(ConflictResolution.RenameAll)}
              title="すべての競合でリネームします"
            >
              すべてリネーム (Rename All)
            </button>
          </div>

          {/* キャンセルボタン */}
          {onCancel && (
            <div className="action-group cancel-action">
              <button
                className="btn btn-secondary"
                onClick={onCancel}
              >
                キャンセル (Cancel)
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ConflictDialog;
