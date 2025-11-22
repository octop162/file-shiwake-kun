/**
 * PreviewDialog コンポーネントのテスト
 * Tests for PreviewDialog component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import PreviewDialog from './PreviewDialog';
import { ProcessResult } from '../types';

describe('PreviewDialog', () => {
  const mockPreviewResults: ProcessResult[] = [
    {
      source_path: '/test/file1.jpg',
      destination_path: '/dest/2024/01/file1.jpg',
      success: true,
      error_message: null,
      matched_rule: 'Photo Rule',
    },
    {
      source_path: '/test/file2.txt',
      destination_path: null,
      success: false,
      error_message: null,
      matched_rule: null,
    },
  ];

  it('プレビュー結果を表示する', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByText('プレビュー結果')).toBeInTheDocument();
    expect(screen.getByText('file1.jpg')).toBeInTheDocument();
    expect(screen.getByText('file2.txt')).toBeInTheDocument();
  });

  it('サマリー情報を表示する', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByText('合計ファイル数:')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('ルールマッチ:')).toBeInTheDocument();
    const onesElements = screen.getAllByText('1');
    expect(onesElements.length).toBe(2); // ルールマッチとルール未マッチの両方
    expect(screen.getByText('ルール未マッチ:')).toBeInTheDocument();
  });

  it('マッチしたルールを表示する', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByText('Photo Rule')).toBeInTheDocument();
  });

  it('確認ボタンをクリックするとonConfirmが呼ばれる', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    const confirmButton = screen.getByText('実行');
    fireEvent.click(confirmButton);

    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('キャンセルボタンをクリックするとonCancelが呼ばれる', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    const cancelButton = screen.getByText('キャンセル');
    fireEvent.click(cancelButton);

    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('移動先パスを表示する', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByText('/dest/2024/01/file1.jpg')).toBeInTheDocument();
  });

  it('ルール未マッチの場合は適切なメッセージを表示する', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <PreviewDialog
        previewResults={mockPreviewResults}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByText('移動先なし（ルール未マッチ）')).toBeInTheDocument();
  });
});
