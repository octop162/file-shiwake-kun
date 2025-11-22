/**
 * ConflictDialogコンポーネントのユニットテスト
 * Unit tests for ConflictDialog component
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ConflictDialog from './ConflictDialog';
import { FileInfo, ConflictResolution } from '../types';

describe('ConflictDialog Component', () => {
  let mockSourceFile: FileInfo;
  let mockDestFile: FileInfo;
  let onResolveMock: ReturnType<typeof vi.fn>;
  let onCancelMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    // Mock file information
    mockSourceFile = {
      name: 'test-image.jpg',
      size: 1024 * 1024 * 2.5, // 2.5 MB
      mod_time: new Date('2024-01-15T10:30:00').getTime(),
    };

    mockDestFile = {
      name: 'test-image.jpg',
      size: 1024 * 1024 * 3.2, // 3.2 MB
      mod_time: new Date('2024-01-10T08:15:00').getTime(),
    };

    onResolveMock = vi.fn();
    onCancelMock = vi.fn();
  });

  describe('ダイアログ表示のテスト', () => {
    it('should render dialog with conflict warning', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('⚠️ ファイル競合')).toBeInTheDocument();
      expect(screen.getByText('File Conflict Detected')).toBeInTheDocument();
      expect(screen.getByText(/移動先に同名のファイルが既に存在します/)).toBeInTheDocument();
    });

    it('should display source file information correctly', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('📄 移動元ファイル')).toBeInTheDocument();
      expect(screen.getAllByText('test-image.jpg').length).toBeGreaterThan(0);
      expect(screen.getByText('2.50 MB')).toBeInTheDocument();
    });

    it('should display destination file information correctly', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('📁 移動先ファイル')).toBeInTheDocument();
      expect(screen.getByText('3.20 MB')).toBeInTheDocument();
    });

    it('should display formatted file sizes correctly', () => {
      const smallFile: FileInfo = {
        name: 'small.txt',
        size: 512, // 512 B
        mod_time: Date.now(),
      };

      const largeFile: FileInfo = {
        name: 'large.zip',
        size: 1024 * 1024 * 1024 * 1.5, // 1.5 GB
        mod_time: Date.now(),
      };

      render(
        <ConflictDialog
          sourceFile={smallFile}
          destFile={largeFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('512 B')).toBeInTheDocument();
      expect(screen.getByText('1.50 GB')).toBeInTheDocument();
    });

    it('should display formatted modification times', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      // Check that dates are displayed (format may vary by locale)
      const dateElements = screen.getAllByText(/2024/);
      expect(dateElements.length).toBeGreaterThan(0);
    });

    it('should display "apply to all" checkbox', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toBeInTheDocument();
      expect(checkbox).not.toBeChecked();
      expect(screen.getByText(/以降も同様に処理する/)).toBeInTheDocument();
    });

    it('should display all resolution buttons', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      // Individual actions
      expect(screen.getByText(/上書き \(Overwrite\)/)).toBeInTheDocument();
      expect(screen.getByText(/スキップ \(Skip\)/)).toBeInTheDocument();
      expect(screen.getByText(/リネーム \(Rename\)/)).toBeInTheDocument();

      // Batch actions
      expect(screen.getByText(/すべて上書き \(Overwrite All\)/)).toBeInTheDocument();
      expect(screen.getByText(/すべてスキップ \(Skip All\)/)).toBeInTheDocument();
      expect(screen.getByText(/すべてリネーム \(Rename All\)/)).toBeInTheDocument();
    });

    it('should display cancel button when onCancel is provided', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
          onCancel={onCancelMock}
        />
      );

      expect(screen.getByText(/キャンセル \(Cancel\)/)).toBeInTheDocument();
    });

    it('should not display cancel button when onCancel is not provided', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.queryByText(/キャンセル \(Cancel\)/)).not.toBeInTheDocument();
    });
  });

  describe('解決オプション選択のテスト', () => {
    it('should call onResolve with Overwrite when overwrite button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const overwriteButton = screen.getByText(/上書き \(Overwrite\)/);
      fireEvent.click(overwriteButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.Overwrite, false);
    });

    it('should call onResolve with Skip when skip button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const skipButton = screen.getByText(/スキップ \(Skip\)/);
      fireEvent.click(skipButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.Skip, false);
    });

    it('should call onResolve with Rename when rename button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const renameButton = screen.getByText(/リネーム \(Rename\)/);
      fireEvent.click(renameButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.Rename, false);
    });

    it('should call onResolve with OverwriteAll when overwrite all button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const overwriteAllButton = screen.getByText(/すべて上書き \(Overwrite All\)/);
      fireEvent.click(overwriteAllButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.OverwriteAll, false);
    });

    it('should call onResolve with SkipAll when skip all button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const skipAllButton = screen.getByText(/すべてスキップ \(Skip All\)/);
      fireEvent.click(skipAllButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.SkipAll, false);
    });

    it('should call onResolve with RenameAll when rename all button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const renameAllButton = screen.getByText(/すべてリネーム \(Rename All\)/);
      fireEvent.click(renameAllButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.RenameAll, false);
    });

    it('should call onResolve with applyToAll=true when checkbox is checked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      // Check the "apply to all" checkbox
      const checkbox = screen.getByRole('checkbox');
      fireEvent.click(checkbox);
      expect(checkbox).toBeChecked();

      // Click overwrite button
      const overwriteButton = screen.getByText(/上書き \(Overwrite\)/);
      fireEvent.click(overwriteButton);

      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.Overwrite, true);
    });

    it('should toggle checkbox state correctly', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      const checkbox = screen.getByRole('checkbox');
      
      // Initially unchecked
      expect(checkbox).not.toBeChecked();

      // Check it
      fireEvent.click(checkbox);
      expect(checkbox).toBeChecked();

      // Uncheck it
      fireEvent.click(checkbox);
      expect(checkbox).not.toBeChecked();
    });

    it('should call onCancel when cancel button is clicked', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
          onCancel={onCancelMock}
        />
      );

      const cancelButton = screen.getByText(/キャンセル \(Cancel\)/);
      fireEvent.click(cancelButton);

      expect(onCancelMock).toHaveBeenCalled();
      expect(onResolveMock).not.toHaveBeenCalled();
    });

    it('should pass applyToAll state with batch actions', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      // Check the "apply to all" checkbox
      const checkbox = screen.getByRole('checkbox');
      fireEvent.click(checkbox);

      // Click a batch action
      const overwriteAllButton = screen.getByText(/すべて上書き \(Overwrite All\)/);
      fireEvent.click(overwriteAllButton);

      // Even with checkbox checked, batch actions should pass the checkbox state
      expect(onResolveMock).toHaveBeenCalledWith(ConflictResolution.OverwriteAll, true);
    });

    it('should handle multiple button clicks correctly', () => {
      render(
        <ConflictDialog
          sourceFile={mockSourceFile}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      // Click overwrite
      const overwriteButton = screen.getByText(/上書き \(Overwrite\)/);
      fireEvent.click(overwriteButton);
      expect(onResolveMock).toHaveBeenCalledTimes(1);

      // Click skip
      const skipButton = screen.getByText(/スキップ \(Skip\)/);
      fireEvent.click(skipButton);
      expect(onResolveMock).toHaveBeenCalledTimes(2);

      // Verify the calls
      expect(onResolveMock).toHaveBeenNthCalledWith(1, ConflictResolution.Overwrite, false);
      expect(onResolveMock).toHaveBeenNthCalledWith(2, ConflictResolution.Skip, false);
    });
  });

  describe('ファイルサイズフォーマットのテスト', () => {
    it('should format bytes correctly', () => {
      const file: FileInfo = {
        name: 'tiny.txt',
        size: 500,
        mod_time: Date.now(),
      };

      render(
        <ConflictDialog
          sourceFile={file}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('500 B')).toBeInTheDocument();
    });

    it('should format kilobytes correctly', () => {
      const file: FileInfo = {
        name: 'small.txt',
        size: 1024 * 50.5, // 50.5 KB
        mod_time: Date.now(),
      };

      render(
        <ConflictDialog
          sourceFile={file}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('50.50 KB')).toBeInTheDocument();
    });

    it('should format megabytes correctly', () => {
      const file: FileInfo = {
        name: 'medium.jpg',
        size: 1024 * 1024 * 15.75, // 15.75 MB
        mod_time: Date.now(),
      };

      render(
        <ConflictDialog
          sourceFile={file}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('15.75 MB')).toBeInTheDocument();
    });

    it('should format gigabytes correctly', () => {
      const file: FileInfo = {
        name: 'large.zip',
        size: 1024 * 1024 * 1024 * 2.25, // 2.25 GB
        mod_time: Date.now(),
      };

      render(
        <ConflictDialog
          sourceFile={file}
          destFile={mockDestFile}
          onResolve={onResolveMock}
        />
      );

      expect(screen.getByText('2.25 GB')).toBeInTheDocument();
    });
  });
});
