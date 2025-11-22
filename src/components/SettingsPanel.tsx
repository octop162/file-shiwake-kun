/**
 * 設定パネルコンポーネント
 * Settings panel component for managing rules
 */

import { useApp } from '../context';
import { Rule } from '../types';

interface SettingsPanelProps {
  onAddRule?: (rule: Rule) => void;
  onEditRule?: (index: number, rule: Rule) => void;
  onDeleteRule?: (index: number) => void;
  onReorderRules?: (from: number, to: number) => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = (props) => {
  const { config } = useApp();
  
  // Props will be used in task 13
  console.log('Settings panel props for future use:', props);

  if (!config) {
    return <div>設定を読み込み中...</div>;
  }

  return (
    <div className="settings-panel">
      <h2>設定</h2>
      
      <div className="settings-section">
        <h3>整理ルール</h3>
        <p>ルール数: {config.rules.length}</p>
        
        {config.rules.length === 0 ? (
          <p>ルールがありません</p>
        ) : (
          <ul className="rules-list">
            {config.rules.map((rule) => (
              <li key={rule.id}>
                <strong>{rule.name}</strong> (優先度: {rule.priority})
                <br />
                <small>移動先: {rule.destination_pattern}</small>
              </li>
            ))}
          </ul>
        )}
        
        {/* TODO: Implement rule management UI in task 13 */}
      </div>

      <div className="settings-section">
        <h3>一般設定</h3>
        <p>デフォルト移動先: {config.default_destination}</p>
        <p>プレビューモード: {config.preview_mode ? 'ON' : 'OFF'}</p>
        <p>ログパス: {config.log_path}</p>
      </div>
    </div>
  );
};

export default SettingsPanel;
