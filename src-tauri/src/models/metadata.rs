use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    // ファイルシステム属性
    pub filename: String,
    pub extension: String,
    pub size: u64,
    pub created_at: Option<SystemTime>,
    pub modified_at: SystemTime,
    
    // EXIF情報（画像ファイルの場合）
    pub capture_date: Option<SystemTime>,
    pub camera_model: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
}
