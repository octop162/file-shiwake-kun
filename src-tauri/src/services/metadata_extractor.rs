use crate::models::FileMetadata;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub trait MetadataExtractor: Send + Sync {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String>;
}

pub struct DefaultMetadataExtractor;

impl MetadataExtractor for DefaultMetadataExtractor {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String> {
        let path = Path::new(filepath);
        
        // ファイルが存在するか確認
        if !path.exists() {
            return Err(format!("File does not exist: {}", filepath));
        }
        
        // ファイルシステムメタデータを抽出
        let fs_metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        // ファイル名と拡張子を取得
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", filepath))?
            .to_string();
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // ファイルサイズを取得
        let size = fs_metadata.len();
        
        // 作成日時を取得（プラットフォームによっては利用できない場合がある）
        let created_at = fs_metadata.created().ok();
        
        // 更新日時を取得
        let modified_at = fs_metadata.modified()
            .map_err(|e| format!("Failed to read modification time: {}", e))?;
        
        // 画像ファイルの場合、EXIF情報を抽出
        let (capture_date, camera_model, gps_latitude, gps_longitude) = 
            if is_image_file(&extension) {
                extract_exif_metadata(path)
            } else {
                (None, None, None, None)
            };
        
        Ok(FileMetadata {
            filename,
            extension,
            size,
            created_at,
            modified_at,
            capture_date,
            camera_model,
            gps_latitude,
            gps_longitude,
        })
    }
}

/// 画像ファイルかどうかを判定
pub(crate) fn is_image_file(extension: &str) -> bool {
    matches!(
        extension,
        "jpg" | "jpeg" | "png" | "heic" | "heif" | "tif" | "tiff" | 
        "cr2" | "nef" | "arw" | "dng" | "orf" | "rw2" | "raf"
    )
}

/// EXIF情報を抽出
fn extract_exif_metadata(path: &Path) -> (Option<SystemTime>, Option<String>, Option<f64>, Option<f64>) {
    // EXIF抽出を試みる。失敗してもエラーにはせず、Noneを返す
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None, None, None),
    };
    
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(reader) => reader,
        Err(_) => return (None, None, None, None),
    };
    
    // 撮影日時を取得
    let capture_date = extract_capture_date(&exifreader);
    
    // カメラ機種を取得
    let camera_model = extract_camera_model(&exifreader);
    
    // GPS座標を取得
    let (gps_latitude, gps_longitude) = extract_gps_coordinates(&exifreader);
    
    (capture_date, camera_model, gps_latitude, gps_longitude)
}

/// EXIF情報から撮影日時を抽出
fn extract_capture_date(exifreader: &exif::Exif) -> Option<SystemTime> {
    // DateTimeOriginal（撮影日時）を優先的に取得
    let datetime_field = exifreader.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .or_else(|| exifreader.get_field(exif::Tag::DateTime, exif::In::PRIMARY))?;
    
    let datetime_str = datetime_field.display_value().to_string();
    
    // EXIF日時形式: "YYYY:MM:DD HH:MM:SS"
    parse_exif_datetime(&datetime_str)
}

/// EXIF日時文字列をSystemTimeに変換
fn parse_exif_datetime(datetime_str: &str) -> Option<SystemTime> {
    // "YYYY:MM:DD HH:MM:SS" 形式をパース
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    
    let date_parts: Vec<&str> = parts[0].split(':').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }
    
    let year: i32 = date_parts[0].parse().ok()?;
    let month: u32 = date_parts[1].parse().ok()?;
    let day: u32 = date_parts[2].parse().ok()?;
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts[2].parse().ok()?;
    
    // 簡易的な日時からUNIXタイムスタンプへの変換
    // 1970年1月1日からの経過秒数を計算
    let days_since_epoch = days_from_civil(year, month, day)?;
    let seconds = days_since_epoch as u64 * 86400 + hour as u64 * 3600 + minute as u64 * 60 + second as u64;
    
    Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(seconds))
}

/// 年月日から1970年1月1日からの経過日数を計算
fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    
    Some(era * 146097 + doe as i64 - 719468)
}

/// EXIF情報からカメラ機種を抽出
fn extract_camera_model(exifreader: &exif::Exif) -> Option<String> {
    let model_field = exifreader.get_field(exif::Tag::Model, exif::In::PRIMARY)?;
    Some(model_field.display_value().to_string())
}

/// EXIF情報からGPS座標を抽出
fn extract_gps_coordinates(exifreader: &exif::Exif) -> (Option<f64>, Option<f64>) {
    let latitude = extract_gps_coordinate(
        exifreader,
        exif::Tag::GPSLatitude,
        exif::Tag::GPSLatitudeRef,
    );
    
    let longitude = extract_gps_coordinate(
        exifreader,
        exif::Tag::GPSLongitude,
        exif::Tag::GPSLongitudeRef,
    );
    
    (latitude, longitude)
}

/// GPS座標を抽出（緯度または経度）
fn extract_gps_coordinate(
    exifreader: &exif::Exif,
    coord_tag: exif::Tag,
    ref_tag: exif::Tag,
) -> Option<f64> {
    let coord_field = exifreader.get_field(coord_tag, exif::In::PRIMARY)?;
    let ref_field = exifreader.get_field(ref_tag, exif::In::PRIMARY)?;
    
    // GPS座標は通常、度・分・秒の3つの有理数として格納されている
    let coord_value = match &coord_field.value {
        exif::Value::Rational(rationals) if rationals.len() >= 3 => {
            let degrees = rationals[0].to_f64();
            let minutes = rationals[1].to_f64();
            let seconds = rationals[2].to_f64();
            degrees + minutes / 60.0 + seconds / 3600.0
        }
        _ => return None,
    };
    
    // 参照方向（N/S または E/W）を確認
    let ref_str = ref_field.display_value().to_string();
    let multiplier = if ref_str.contains('S') || ref_str.contains('W') {
        -1.0
    } else {
        1.0
    };
    
    Some(coord_value * multiplier)
}
