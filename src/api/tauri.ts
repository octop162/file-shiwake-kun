/**
 * Tauri API Client
 * Provides typed wrappers for Tauri commands with error handling
 */

import { invoke } from '@tauri-apps/api/core';
import { Config, ProcessResult, FileInfo } from '../types';
import { logger } from '../utils/logger';
import { parseErrorMessage } from '../utils/errorRecovery';

/**
 * ファイル処理を開始
 * Process files according to configured rules
 * Requirement 8.1, 8.5: Log all file operations
 */
export async function processFiles(files: string[]): Promise<ProcessResult[]> {
  try {
    logger.info('Invoking process_files command', { fileCount: files.length });
    const results = await invoke<ProcessResult[]>('process_files', { files });
    logger.info('process_files command completed', { resultCount: results.length });
    return results;
  } catch (error) {
    const errorMessage = parseErrorMessage(error);
    logger.error('process_files command failed', { error: errorMessage });
    throw new Error(errorMessage);
  }
}

/**
 * 設定を読み込む
 * Load configuration from TOML file
 * Requirement 5.1: Load config from TOML file on startup
 * Requirement 5.4: Handle invalid TOML gracefully
 */
export async function loadConfig(): Promise<Config> {
  try {
    logger.info('Invoking load_config command');
    const config = await invoke<Config>('load_config');
    logger.info('load_config command completed', { rulesCount: config.rules.length });
    return config;
  } catch (error) {
    const errorMessage = parseErrorMessage(error);
    logger.error('load_config command failed', { error: errorMessage });
    throw new Error(errorMessage);
  }
}

/**
 * 設定を保存
 * Save configuration to TOML file
 * Requirement 5.2: Save changes to TOML file
 */
export async function saveConfig(config: Config): Promise<void> {
  try {
    logger.info('Invoking save_config command', { rulesCount: config.rules.length });
    await invoke<void>('save_config', { config });
    logger.info('save_config command completed');
  } catch (error) {
    const errorMessage = parseErrorMessage(error);
    logger.error('save_config command failed', { error: errorMessage });
    throw new Error(errorMessage);
  }
}

/**
 * ファイル情報を取得
 * Get file information
 */
export async function getFileInfo(path: string): Promise<FileInfo> {
  try {
    logger.debug('Invoking get_file_info command', { path });
    const fileInfo = await invoke<FileInfo>('get_file_info', { path });
    logger.debug('get_file_info command completed', { fileName: fileInfo.name });
    return fileInfo;
  } catch (error) {
    const errorMessage = parseErrorMessage(error);
    logger.error('get_file_info command failed', { error: errorMessage, path });
    throw new Error(errorMessage);
  }
}
