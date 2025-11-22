/**
 * メインウィンドウコンポーネント
 * Main window component with drag & drop functionality
 */

import { useApp } from '../context';

interface MainWindowProps {
  onFileDrop: (files: string[]) => void;
}

const MainWindow: React.FC<MainWindowProps> = ({ onFileDrop }) => {
  const { config, isProcessing, processResults } = useApp();

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    // TODO: Extract file paths from drop event and call onFileDrop
    // This will be implemented in task 12
    console.log('File drop handler - to be implemented', onFileDrop);
  };

  return (
    <div className="main-window">
      <h1>ファイル仕訳け君</h1>
      
      <div 
        className="drop-zone"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <p>ファイルをここにドラッグ&ドロップ</p>
        <p className="hint">Drag and drop files here</p>
      </div>

      {isProcessing && (
        <div className="processing-indicator">
          <p>処理中...</p>
        </div>
      )}

      {processResults.length > 0 && (
        <div className="results">
          <h2>処理結果</h2>
          <ul>
            {processResults.map((result, index) => (
              <li key={index}>
                {result.success ? '✓' : '✗'} {result.source_path}
                {result.error_message && <span className="error"> - {result.error_message}</span>}
              </li>
            ))}
          </ul>
        </div>
      )}

      {config && (
        <div className="config-info">
          <p>ルール数: {config.rules.length}</p>
          <p>プレビューモード: {config.preview_mode ? 'ON' : 'OFF'}</p>
        </div>
      )}
    </div>
  );
};

export default MainWindow;
