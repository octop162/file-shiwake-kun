/**
 * Tauri API Client
 * Provides typed wrappers for Tauri commands
 */

import { invoke } from '@tauri-apps/api/core';
import { Config, ProcessResult, FileInfo } from '../types';

/**
 * ファイル処理を開始
 * Process files according to configured rules
 */
export async function processFiles(files: string[]): Promise<ProcessResult[]> {
  return await invoke<ProcessResult[]>('process_files', { files });
}

/**
 * 設定を読み込む
 * Load configuration from TOML file
 */
export async function loadConfig(): Promise<Config> {
  return await invoke<Config>('load_config');
}

/**
 * 設定を保存
 * Save configuration to TOML file
 */
export async function saveConfig(config: Config): Promise<void> {
  return await invoke<void>('save_config', { config });
}

/**
 * ファイル情報を取得
 * Get file information
 */
export async function getFileInfo(path: string): Promise<FileInfo> {
  return await invoke<FileInfo>('get_file_info', { path });
}
