/**
 * TypeScript type definitions matching Rust data models
 * These types are used for communication between the React frontend and Tauri backend
 */

/**
 * ファイル操作の種類
 * File operation type
 */
export enum OperationType {
  Move = "Move",
  Copy = "Copy",
}

/**
 * ルールの条件
 * Rule condition for matching files
 */
export interface Condition {
  field: string;
  operator: string;
  value: any; // Corresponds to serde_json::Value
}

/**
 * ファイル整理ルール
 * File organization rule
 */
export interface Rule {
  id: string;
  name: string;
  priority: number;
  conditions: Condition[];
  destination_pattern: string;
  operation: OperationType;
}

/**
 * アプリケーション設定
 * Application configuration
 */
export interface Config {
  rules: Rule[];
  default_destination: string;
  preview_mode: boolean;
  log_path: string;
}

/**
 * ファイルメタデータ
 * File metadata including filesystem attributes and EXIF data
 */
export interface FileMetadata {
  // ファイルシステム属性
  filename: string;
  extension: string;
  size: number;
  created_at: number | null; // SystemTime as Unix timestamp (milliseconds)
  modified_at: number; // SystemTime as Unix timestamp (milliseconds)
  
  // EXIF情報（画像ファイルの場合）
  capture_date: number | null; // SystemTime as Unix timestamp (milliseconds)
  camera_model: string | null;
  gps_latitude: number | null;
  gps_longitude: number | null;
}

/**
 * ファイル処理結果
 * Result of file processing operation
 */
export interface ProcessResult {
  source_path: string;
  destination_path: string | null;
  success: boolean;
  error_message: string | null;
  matched_rule: string | null;
}

/**
 * ファイル情報
 * Basic file information
 */
export interface FileInfo {
  name: string;
  size: number;
  mod_time: number; // SystemTime as Unix timestamp (milliseconds)
}

/**
 * ファイル競合時の解決方法
 * Conflict resolution strategy when destination file exists
 */
export enum ConflictResolution {
  Overwrite = "Overwrite",
  Skip = "Skip",
  Rename = "Rename",
  OverwriteAll = "OverwriteAll",
  SkipAll = "SkipAll",
  RenameAll = "RenameAll",
}
