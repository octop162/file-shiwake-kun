/**
 * 設定パネルコンポーネント
 * Settings panel component for managing rules
 */

import React, { useState } from 'react';
import { useApp } from '../context';
import { Rule, Condition, OperationType } from '../types';

interface SettingsPanelProps {
  onAddRule?: (rule: Rule) => void;
  onEditRule?: (index: number, rule: Rule) => void;
  onDeleteRule?: (index: number) => void;
  onReorderRules?: (from: number, to: number) => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = (props) => {
  const { config, updateConfig } = useApp();
  const [isAddingRule, setIsAddingRule] = useState(false);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  
  // Form state
  const [formData, setFormData] = useState<{
    name: string;
    priority: number;
    destination_pattern: string;
    operation: OperationType;
    conditions: Condition[];
  }>({
    name: '',
    priority: 1,
    destination_pattern: '',
    operation: OperationType.Move,
    conditions: [],
  });

  if (!config) {
    return <div>設定を読み込み中...</div>;
  }

  const resetForm = () => {
    setFormData({
      name: '',
      priority: 1,
      destination_pattern: '',
      operation: OperationType.Move,
      conditions: [],
    });
    setIsAddingRule(false);
    setEditingIndex(null);
  };

  const handleAddRule = async () => {
    const newRule: Rule = {
      id: `rule-${Date.now()}`,
      name: formData.name,
      priority: formData.priority,
      conditions: formData.conditions,
      destination_pattern: formData.destination_pattern,
      operation: formData.operation,
    };

    const newConfig = {
      ...config,
      rules: [...config.rules, newRule],
    };

    try {
      await updateConfig(newConfig);
      props.onAddRule?.(newRule);
      resetForm();
    } catch (error) {
      console.error('Failed to add rule:', error);
    }
  };

  const handleEditRule = async (index: number) => {
    const rule = config.rules[index];
    setFormData({
      name: rule.name,
      priority: rule.priority,
      destination_pattern: rule.destination_pattern,
      operation: rule.operation,
      conditions: rule.conditions,
    });
    setEditingIndex(index);
    setIsAddingRule(true);
  };

  const handleSaveEdit = async () => {
    if (editingIndex === null) return;

    const updatedRule: Rule = {
      id: config.rules[editingIndex].id,
      name: formData.name,
      priority: formData.priority,
      conditions: formData.conditions,
      destination_pattern: formData.destination_pattern,
      operation: formData.operation,
    };

    const newRules = [...config.rules];
    newRules[editingIndex] = updatedRule;

    const newConfig = {
      ...config,
      rules: newRules,
    };

    try {
      await updateConfig(newConfig);
      props.onEditRule?.(editingIndex, updatedRule);
      resetForm();
    } catch (error) {
      console.error('Failed to edit rule:', error);
    }
  };

  const handleDeleteRule = async (index: number) => {
    if (!confirm('このルールを削除してもよろしいですか？')) return;

    const newRules = config.rules.filter((_, i) => i !== index);
    const newConfig = {
      ...config,
      rules: newRules,
    };

    try {
      await updateConfig(newConfig);
      props.onDeleteRule?.(index);
    } catch (error) {
      console.error('Failed to delete rule:', error);
    }
  };

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDrop = async (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex) return;

    const newRules = [...config.rules];
    const [draggedRule] = newRules.splice(draggedIndex, 1);
    newRules.splice(dropIndex, 0, draggedRule);

    const newConfig = {
      ...config,
      rules: newRules,
    };

    try {
      await updateConfig(newConfig);
      props.onReorderRules?.(draggedIndex, dropIndex);
      setDraggedIndex(null);
    } catch (error) {
      console.error('Failed to reorder rules:', error);
    }
  };

  const handleAddCondition = () => {
    setFormData({
      ...formData,
      conditions: [
        ...formData.conditions,
        { field: 'extension', operator: '==', value: '' },
      ],
    });
  };

  const handleUpdateCondition = (index: number, field: keyof Condition, value: any) => {
    const newConditions = [...formData.conditions];
    newConditions[index] = { ...newConditions[index], [field]: value };
    setFormData({ ...formData, conditions: newConditions });
  };

  const handleRemoveCondition = (index: number) => {
    const newConditions = formData.conditions.filter((_, i) => i !== index);
    setFormData({ ...formData, conditions: newConditions });
  };

  return (
    <div className="settings-panel">
      <h2>設定</h2>
      
      <div className="settings-section">
        <div className="section-header">
          <h3>整理ルール</h3>
          {!isAddingRule && (
            <button 
              className="btn btn-primary"
              onClick={() => setIsAddingRule(true)}
            >
              ＋ ルールを追加
            </button>
          )}
        </div>

        {isAddingRule && (
          <div className="rule-form">
            <h4>{editingIndex !== null ? 'ルールを編集' : '新しいルールを追加'}</h4>
            
            <div className="form-group">
              <label>ルール名</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="例: 写真を年月別に整理"
              />
            </div>

            <div className="form-group">
              <label>優先度</label>
              <input
                type="number"
                value={formData.priority}
                onChange={(e) => setFormData({ ...formData, priority: parseInt(e.target.value) })}
                min="1"
              />
            </div>

            <div className="form-group">
              <label>移動先パターン</label>
              <input
                type="text"
                value={formData.destination_pattern}
                onChange={(e) => setFormData({ ...formData, destination_pattern: e.target.value })}
                placeholder="例: D:/Photos/{year}/{month}"
              />
              <small className="hint">
                使用可能な変数: {'{year}'}, {'{month}'}, {'{day}'}, {'{extension}'}, {'{camera}'}, {'{filename}'}
              </small>
            </div>

            <div className="form-group">
              <label>操作</label>
              <select
                value={formData.operation}
                onChange={(e) => setFormData({ ...formData, operation: e.target.value as OperationType })}
              >
                <option value={OperationType.Move}>移動</option>
                <option value={OperationType.Copy}>コピー</option>
              </select>
            </div>

            <div className="form-group">
              <div className="conditions-header">
                <label>条件</label>
                <button 
                  type="button"
                  className="btn btn-small"
                  onClick={handleAddCondition}
                >
                  ＋ 条件を追加
                </button>
              </div>
              
              {formData.conditions.map((condition, index) => (
                <div key={index} className="condition-row">
                  <select
                    value={condition.field}
                    onChange={(e) => handleUpdateCondition(index, 'field', e.target.value)}
                  >
                    <option value="extension">拡張子</option>
                    <option value="size">ファイルサイズ</option>
                    <option value="capture_date">撮影日時</option>
                    <option value="camera_model">カメラ機種</option>
                    <option value="created_at">作成日時</option>
                    <option value="modified_at">更新日時</option>
                  </select>

                  <select
                    value={condition.operator}
                    onChange={(e) => handleUpdateCondition(index, 'operator', e.target.value)}
                  >
                    <option value="==">=</option>
                    <option value="!=">≠</option>
                    <option value="in">含む</option>
                    <option value="exists">存在する</option>
                    <option value=">">＞</option>
                    <option value="<">＜</option>
                  </select>

                  <input
                    type="text"
                    value={typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value)}
                    onChange={(e) => {
                      let value: any = e.target.value;
                      try {
                        value = JSON.parse(e.target.value);
                      } catch {
                        // Keep as string if not valid JSON
                      }
                      handleUpdateCondition(index, 'value', value);
                    }}
                    placeholder="値"
                  />

                  <button
                    type="button"
                    className="btn btn-danger btn-small"
                    onClick={() => handleRemoveCondition(index)}
                  >
                    ✕
                  </button>
                </div>
              ))}
            </div>

            <div className="form-actions">
              <button 
                className="btn btn-primary"
                onClick={editingIndex !== null ? handleSaveEdit : handleAddRule}
                disabled={!formData.name || !formData.destination_pattern}
              >
                {editingIndex !== null ? '保存' : '追加'}
              </button>
              <button 
                className="btn btn-secondary"
                onClick={resetForm}
              >
                キャンセル
              </button>
            </div>
          </div>
        )}

        <div className="rules-list-container">
          {config.rules.length === 0 ? (
            <p className="empty-message">ルールがありません。「ルールを追加」ボタンから新しいルールを作成してください。</p>
          ) : (
            <ul className="rules-list">
              {config.rules.map((rule, index) => (
                <li
                  key={rule.id}
                  draggable
                  onDragStart={() => handleDragStart(index)}
                  onDragOver={(e) => handleDragOver(e)}
                  onDrop={(e) => handleDrop(e, index)}
                  className={draggedIndex === index ? 'dragging' : ''}
                >
                  <div className="rule-header">
                    <div className="rule-drag-handle">⋮⋮</div>
                    <div className="rule-info">
                      <strong>{rule.name}</strong>
                      <span className="rule-priority">優先度: {rule.priority}</span>
                    </div>
                    <div className="rule-actions">
                      <button
                        className="btn btn-small btn-edit"
                        onClick={() => handleEditRule(index)}
                        title="編集"
                      >
                        ✎
                      </button>
                      <button
                        className="btn btn-small btn-danger"
                        onClick={() => handleDeleteRule(index)}
                        title="削除"
                      >
                        ✕
                      </button>
                    </div>
                  </div>
                  <div className="rule-details">
                    <div className="rule-detail">
                      <span className="label">操作:</span>
                      <span className="value">{rule.operation === OperationType.Move ? '移動' : 'コピー'}</span>
                    </div>
                    <div className="rule-detail">
                      <span className="label">移動先:</span>
                      <span className="value">{rule.destination_pattern}</span>
                    </div>
                    {rule.conditions.length > 0 && (
                      <div className="rule-detail">
                        <span className="label">条件:</span>
                        <span className="value">{rule.conditions.length}件</span>
                      </div>
                    )}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      <div className="settings-section">
        <h3>一般設定</h3>
        
        <div className="form-group">
          <label className="toggle-label">
            <span>プレビューモード</span>
            <div className="toggle-description">
              有効にすると、ファイル操作を実行する前にプレビューを表示します
            </div>
          </label>
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={config.preview_mode}
              onChange={async (e) => {
                const newConfig = {
                  ...config,
                  preview_mode: e.target.checked,
                };
                try {
                  await updateConfig(newConfig);
                } catch (error) {
                  console.error('Failed to update preview mode:', error);
                }
              }}
            />
            <span className="toggle-slider"></span>
          </label>
        </div>

        <div className="config-details">
          <p>
            <span className="config-label">デフォルト移動先:</span>
            <span className="config-value">{config.default_destination}</span>
          </p>
          <p>
            <span className="config-label">ログパス:</span>
            <span className="config-value">{config.log_path}</span>
          </p>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
