/**
 * プレビューダイアログコンポーネント
 * Preview dialog component for showing file processing preview
 */

import React from 'react';
import { ProcessResult } from '../types';

interface PreviewDialogProps {
  previewResults: ProcessResult[];
  onConfirm: () => void;
  onCancel: () => void;
}

const PreviewDialog: React.FC<PreviewDialogProps> = ({
  previewResults,
  onConfirm,
  onCancel,
}) => {
  /**
   * プレビュー結果のサマリーを計算
   */
  const getSummary = () => {
    const withRule = previewResults.filter(r => r.matched_rule !== null).length;
    const withoutRule = previewResults.filter(r => r.matched_rule === null).length;
    return { withRule, withoutRule, total: previewResults.length };
  };

  const summary = getSummary();

  return (
    <div className="preview-dialog-overlay">
      <div className="preview-dialog">
        <div className="preview-dialog-header">
          <h2>プレビュー結果</h2>
          <p className="preview-dialog-subtitle">
            以下の操作が実行されます。確認してください。
          </p>
        </div>

        <div className="preview-dialog-summary">
          <div className="summary-item">
            <span className="summary-label">合計ファイル数:</span>
            <span className="summary-value">{summary.total}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">ルールマッチ:</span>
            <span className="summary-value success">{summary.withRule}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">ルール未マッチ:</span>
            <span className="summary-value warning">{summary.withoutRule}</span>
          </div>
        </div>

        <div className="preview-dialog-content">
          <div className="preview-results-list">
            {previewResults.map((result, index) => (
              <div
                key={index}
                className={`preview-result-item ${
                  result.matched_rule ? 'matched' : 'unmatched'
                }`}
              >
                <div className="preview-result-header">
                  <span className="preview-result-icon">
                    {result.matched_rule ? '📄' : '❓'}
                  </span>
                  <span className="preview-result-filename">
                    {result.source_path.split(/[\\/]/).pop()}
                  </span>
                </div>

                <div className="preview-result-details">
                  <div className="preview-result-path">
                    <span className="path-label">元:</span>
                    <span className="path-value">{result.source_path}</span>
                  </div>

                  {result.destination_path ? (
                    <div className="preview-result-path">
                      <span className="path-label">先:</span>
                      <span className="path-value destination">
                        {result.destination_path}
                      </span>
                    </div>
                  ) : (
                    <div className="preview-result-path">
                      <span className="path-label">先:</span>
                      <span className="path-value no-destination">
                        移動先なし（ルール未マッチ）
                      </span>
                    </div>
                  )}

                  {result.matched_rule && (
                    <div className="preview-result-rule">
                      <span className="rule-label">適用ルール:</span>
                      <span className="rule-value">{result.matched_rule}</span>
                    </div>
                  )}

                  {result.error_message && (
                    <div className="preview-result-error">
                      <span className="error-icon">⚠️</span>
                      <span className="error-message">{result.error_message}</span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="preview-dialog-footer">
          <button
            className="preview-dialog-button cancel"
            onClick={onCancel}
          >
            キャンセル
          </button>
          <button
            className="preview-dialog-button confirm"
            onClick={onConfirm}
          >
            実行
          </button>
        </div>
      </div>
    </div>
  );
};

export default PreviewDialog;
