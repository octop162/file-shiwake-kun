/**
 * SettingsPanelコンポーネントのユニットテスト
 * Unit tests for SettingsPanel component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import SettingsPanel from './SettingsPanel';
import { AppProvider } from '../context';
import { OperationType } from '../types';

// Mock the Tauri API
const mockSaveConfig = vi.fn().mockResolvedValue(undefined);
const mockLoadConfig = vi.fn().mockResolvedValue({
  rules: [
    {
      id: 'rule-001',
      name: 'テストルール1',
      priority: 1,
      conditions: [{ field: 'extension', operator: '==', value: '.jpg' }],
      destination_pattern: 'D:/Photos/{year}',
      operation: OperationType.Move,
    },
  ],
  default_destination: 'C:/Test',
  preview_mode: false,
  log_path: 'test.log',
});

vi.mock('../api/tauri', () => ({
  loadConfig: () => mockLoadConfig(),
  saveConfig: (config: any) => mockSaveConfig(config),
  processFiles: vi.fn().mockResolvedValue([]),
}));

/**
 * テスト用のラッパーコンポーネント
 */
const renderWithProvider = (ui: React.ReactElement) => {
  return render(<AppProvider>{ui}</AppProvider>);
};

describe('SettingsPanel Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('ルール追加のテスト', () => {
    it('should display add rule button', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        expect(screen.getByText('＋ ルールを追加')).toBeInTheDocument();
      });
    });

    it('should show rule form when add button is clicked', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      expect(screen.getByText('新しいルールを追加')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('例: 写真を年月別に整理')).toBeInTheDocument();
    });

    it('should add a new rule when form is submitted', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      // Fill in the form
      const nameInput = screen.getByPlaceholderText('例: 写真を年月別に整理');
      fireEvent.change(nameInput, { target: { value: '新しいルール' } });

      const patternInput = screen.getByPlaceholderText('例: D:/Photos/{year}/{month}');
      fireEvent.change(patternInput, { target: { value: 'D:/NewFolder' } });

      // Submit the form
      const submitButton = screen.getByText('追加');
      fireEvent.click(submitButton);

      await waitFor(() => {
        expect(mockSaveConfig).toHaveBeenCalled();
      });
    });

    it('should not submit form with empty required fields', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      const submitButton = screen.getByText('追加');
      expect(submitButton).toBeDisabled();
    });

    it('should close form when cancel button is clicked', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      expect(screen.getByText('新しいルールを追加')).toBeInTheDocument();

      const cancelButton = screen.getByText('キャンセル');
      fireEvent.click(cancelButton);

      expect(screen.queryByText('新しいルールを追加')).not.toBeInTheDocument();
    });
  });

  describe('ルール編集のテスト', () => {
    it('should display edit button for existing rules', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        expect(screen.getByText('テストルール1')).toBeInTheDocument();
      });

      const editButtons = screen.getAllByTitle('編集');
      expect(editButtons.length).toBeGreaterThan(0);
    });

    it('should open edit form when edit button is clicked', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const editButton = screen.getByTitle('編集');
        fireEvent.click(editButton);
      });

      expect(screen.getByText('ルールを編集')).toBeInTheDocument();
      expect(screen.getByDisplayValue('テストルール1')).toBeInTheDocument();
    });

    it('should update rule when edit form is submitted', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const editButton = screen.getByTitle('編集');
        fireEvent.click(editButton);
      });

      const nameInput = screen.getByDisplayValue('テストルール1');
      fireEvent.change(nameInput, { target: { value: '更新されたルール' } });

      const saveButton = screen.getByText('保存');
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(mockSaveConfig).toHaveBeenCalled();
      });
    });
  });

  describe('ルール削除のテスト', () => {
    it('should display delete button for existing rules', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        expect(screen.getByText('テストルール1')).toBeInTheDocument();
      });

      const deleteButtons = screen.getAllByTitle('削除');
      expect(deleteButtons.length).toBeGreaterThan(0);
    });

    it('should show confirmation dialog when delete button is clicked', async () => {
      // Mock window.confirm
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
      
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const deleteButton = screen.getByTitle('削除');
        fireEvent.click(deleteButton);
      });

      expect(confirmSpy).toHaveBeenCalledWith('このルールを削除してもよろしいですか？');
      
      await waitFor(() => {
        expect(mockSaveConfig).toHaveBeenCalled();
      });

      confirmSpy.mockRestore();
    });

    it('should not delete rule when confirmation is cancelled', async () => {
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
      
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const deleteButton = screen.getByTitle('削除');
        fireEvent.click(deleteButton);
      });

      expect(confirmSpy).toHaveBeenCalled();
      expect(mockSaveConfig).not.toHaveBeenCalled();

      confirmSpy.mockRestore();
    });
  });

  describe('条件管理のテスト', () => {
    it('should add condition when add condition button is clicked', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      const addConditionButton = screen.getByText('＋ 条件を追加');
      fireEvent.click(addConditionButton);

      const fieldSelects = screen.getAllByRole('combobox');
      expect(fieldSelects.length).toBeGreaterThan(2); // At least operation + field + operator
    });

    it('should remove condition when remove button is clicked', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);
      });

      // Add two conditions
      const addConditionButton = screen.getByText('＋ 条件を追加');
      fireEvent.click(addConditionButton);
      fireEvent.click(addConditionButton);

      // Get all condition remove buttons (not the cancel button)
      const conditionRows = document.querySelectorAll('.condition-row');
      const initialConditionCount = conditionRows.length;
      expect(initialConditionCount).toBe(2);

      // Click the remove button on the first condition
      const firstConditionRemoveButton = conditionRows[0].querySelector('button');
      if (firstConditionRemoveButton) {
        fireEvent.click(firstConditionRemoveButton);
      }

      // Check that we now have one less condition
      const updatedConditionRows = document.querySelectorAll('.condition-row');
      expect(updatedConditionRows.length).toBe(1);
    });
  });

  describe('一般設定表示のテスト', () => {
    it('should display general settings', async () => {
      renderWithProvider(<SettingsPanel />);
      
      await waitFor(() => {
        expect(screen.getByText('一般設定')).toBeInTheDocument();
        expect(screen.getByText('C:/Test')).toBeInTheDocument();
        expect(screen.getByText('test.log')).toBeInTheDocument();
      });
    });
  });
});
