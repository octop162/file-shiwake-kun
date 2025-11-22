/**
 * ResultsViewer コンポーネントのユニットテスト
 * Unit tests for ResultsViewer component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import ResultsViewer from './ResultsViewer';
import { ProcessResult } from '../types';

describe('ResultsViewer', () => {
  const mockSuccessResult: ProcessResult = {
    source_path: '/test/source/file1.jpg',
    destination_path: '/test/dest/2024/01/file1.jpg',
    success: true,
    error_message: null,
    matched_rule: 'Photos by date',
  };

  const mockFailureResult: ProcessResult = {
    source_path: '/test/source/file2.jpg',
    destination_path: null,
    success: false,
    error_message: 'Permission denied',
    matched_rule: null,
  };

  describe('空の状態', () => {
    it('結果がない場合、空のメッセージを表示する', () => {
      render(<ResultsViewer results={[]} />);
      
      expect(screen.getByText('処理結果はまだありません')).toBeInTheDocument();
      expect(screen.getByText(/ファイルをドラッグ&ドロップして処理を開始してください/)).toBeInTheDocument();
    });
  });

  describe('サマリー表示 - 要件 8.3', () => {
    it('成功と失敗の統計を正しく表示する', () => {
      const results = [
        mockSuccessResult,
        mockFailureResult,
        { ...mockSuccessResult, source_path: '/test/file3.jpg' },
      ];

      render(<ResultsViewer results={results} />);

      // サマリーカードの確認
      expect(screen.getByText('合計')).toBeInTheDocument();
      expect(screen.getByText('3')).toBeInTheDocument();
      expect(screen.getByText('成功')).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument();
      expect(screen.getByText('失敗')).toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument();
    });

    it('成功率を正しく計算して表示する', () => {
      const results = [
        mockSuccessResult,
        mockFailureResult,
        { ...mockSuccessResult, source_path: '/test/file3.jpg' },
        { ...mockSuccessResult, source_path: '/test/file4.jpg' },
      ];

      render(<ResultsViewer results={results} />);

      // 成功率: 3/4 = 75.0%
      expect(screen.getByText('75.0%')).toBeInTheDocument();
    });
  });

  describe('フィルタリング機能', () => {
    const mixedResults = [
      mockSuccessResult,
      mockFailureResult,
      { ...mockSuccessResult, source_path: '/test/file3.jpg' },
    ];

    it('すべてのフィルターがデフォルトで選択されている', () => {
      render(<ResultsViewer results={mixedResults} />);
      
      const allButton = screen.getByRole('button', { name: /すべて \(3\)/ });
      expect(allButton).toHaveClass('active');
    });

    it('成功フィルターをクリックすると成功した結果のみ表示する', () => {
      render(<ResultsViewer results={mixedResults} />);
      
      const successButton = screen.getByRole('button', { name: /成功 \(2\)/ });
      fireEvent.click(successButton);
      
      expect(successButton).toHaveClass('active');
      expect(screen.getByText(/成功のみ/)).toBeInTheDocument();
    });

    it('失敗フィルターをクリックすると失敗した結果のみ表示する', () => {
      render(<ResultsViewer results={mixedResults} />);
      
      const failureButton = screen.getByRole('button', { name: /失敗 \(1\)/ });
      fireEvent.click(failureButton);
      
      expect(failureButton).toHaveClass('active');
      expect(screen.getByText(/失敗のみ/)).toBeInTheDocument();
    });
  });

  describe('処理結果詳細表示 - 要件 8.1, 8.2', () => {
    it('成功した処理の詳細を表示する', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      // ファイル名をクリックして展開
      const filename = screen.getByText('file1.jpg');
      fireEvent.click(filename.closest('.result-header')!);
      
      // 元のパスと移動先パスが表示される (要件 8.1)
      expect(screen.getByText('/test/source/file1.jpg')).toBeInTheDocument();
      expect(screen.getByText('/test/dest/2024/01/file1.jpg')).toBeInTheDocument();
      expect(screen.getByText('Photos by date')).toBeInTheDocument();
    });

    it('失敗した処理のエラーメッセージを表示する', () => {
      render(<ResultsViewer results={[mockFailureResult]} />);
      
      // ファイル名をクリックして展開
      const filename = screen.getByText('file2.jpg');
      fireEvent.click(filename.closest('.result-header')!);
      
      // エラーメッセージが表示される (要件 8.2)
      expect(screen.getByText('Permission denied')).toBeInTheDocument();
    });

    it('結果アイテムをクリックすると展開/折りたたみできる', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      const header = screen.getByText('file1.jpg').closest('.result-header')!;
      
      // 最初は折りたたまれている
      expect(screen.queryByText('/test/source/file1.jpg')).not.toBeInTheDocument();
      
      // クリックして展開
      fireEvent.click(header);
      expect(screen.getByText('/test/source/file1.jpg')).toBeInTheDocument();
      
      // もう一度クリックして折りたたむ
      fireEvent.click(header);
      expect(screen.queryByText('/test/source/file1.jpg')).not.toBeInTheDocument();
    });
  });

  describe('ログビューアー - 要件 8.5', () => {
    it('ログを表示ボタンをクリックするとログビューアーが表示される', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      const showLogButton = screen.getByRole('button', { name: 'ログを表示' });
      fireEvent.click(showLogButton);
      
      expect(screen.getByText('処理ログ')).toBeInTheDocument();
      expect(screen.getByText(/ファイル仕訳け君 処理ログ/)).toBeInTheDocument();
    });

    it('ログビューアーを閉じるボタンが機能する', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      // ログを表示
      const showLogButton = screen.getByRole('button', { name: 'ログを表示' });
      fireEvent.click(showLogButton);
      expect(screen.getByText('処理ログ')).toBeInTheDocument();
      
      // ログを閉じる
      const closeButton = screen.getByRole('button', { name: '×' });
      fireEvent.click(closeButton);
      expect(screen.queryByText('処理ログ')).not.toBeInTheDocument();
    });

    it('ログに処理結果の詳細が含まれる', () => {
      render(<ResultsViewer results={[mockSuccessResult, mockFailureResult]} />);
      
      const showLogButton = screen.getByRole('button', { name: 'ログを表示' });
      fireEvent.click(showLogButton);
      
      const logContent = screen.getByText(/ファイル仕訳け君 処理ログ/).parentElement;
      expect(logContent?.textContent).toContain('合計: 2 件');
      expect(logContent?.textContent).toContain('成功: 1 件');
      expect(logContent?.textContent).toContain('失敗: 1 件');
      expect(logContent?.textContent).toContain('/test/source/file1.jpg');
      expect(logContent?.textContent).toContain('/test/source/file2.jpg');
    });
  });

  describe('ログダウンロード機能', () => {
    it('ログをダウンロードボタンが存在する', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      expect(screen.getByRole('button', { name: 'ログをダウンロード' })).toBeInTheDocument();
    });

    it('ログをダウンロードボタンをクリックするとダウンロードが開始される', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      // DOM APIのモック
      const mockLink = {
        href: '',
        download: '',
        click: vi.fn(),
      };
      const originalCreateElement = document.createElement.bind(document);
      const createElementSpy = vi.spyOn(document, 'createElement').mockImplementation((tagName: string) => {
        if (tagName === 'a') {
          return mockLink as any;
        }
        return originalCreateElement(tagName);
      });
      const appendChildSpy = vi.spyOn(document.body, 'appendChild').mockImplementation((node: any) => node);
      const removeChildSpy = vi.spyOn(document.body, 'removeChild').mockImplementation((node: any) => node);
      
      // URL APIのモック
      const createObjectURLMock = vi.fn(() => 'blob:mock-url');
      const revokeObjectURLMock = vi.fn();
      (globalThis as any).URL.createObjectURL = createObjectURLMock;
      (globalThis as any).URL.revokeObjectURL = revokeObjectURLMock;
      
      const downloadButton = screen.getByRole('button', { name: 'ログをダウンロード' });
      fireEvent.click(downloadButton);
      
      // リンク要素が作成され、クリックされたことを確認
      expect(mockLink.click).toHaveBeenCalled();
      expect(createObjectURLMock).toHaveBeenCalled();
      expect(revokeObjectURLMock).toHaveBeenCalledWith('blob:mock-url');

      createElementSpy.mockRestore();
      appendChildSpy.mockRestore();
      removeChildSpy.mockRestore();
    });
  });

  describe('クリア機能', () => {
    it('クリアボタンが提供されている場合、クリックするとコールバックが呼ばれる', () => {
      const onClear = vi.fn();
      render(<ResultsViewer results={[mockSuccessResult]} onClear={onClear} />);
      
      const clearButton = screen.getByRole('button', { name: 'クリア' });
      fireEvent.click(clearButton);
      
      expect(onClear).toHaveBeenCalledTimes(1);
    });

    it('onClearが提供されていない場合、クリアボタンは表示されない', () => {
      render(<ResultsViewer results={[mockSuccessResult]} />);
      
      expect(screen.queryByRole('button', { name: 'クリア' })).not.toBeInTheDocument();
    });
  });
});
