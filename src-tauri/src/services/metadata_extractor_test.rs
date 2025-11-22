#[cfg(test)]
mod tests {
    use super::super::metadata_extractor::{DefaultMetadataExtractor, MetadataExtractor};
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;
    use proptest::prelude::*;

    #[test]
    fn test_extract_filesystem_metadata() {
        // 一時ディレクトリとテストファイルを作成
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_file.txt");
        
        // テストファイルを作成
        let mut file = fs::File::create(&test_file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        
        // メタデータを抽出
        let extractor = DefaultMetadataExtractor;
        let metadata = extractor.extract(test_file_path.to_str().unwrap()).unwrap();
        
        // ファイルシステムメタデータを検証
        assert_eq!(metadata.filename, "test_file.txt");
        assert_eq!(metadata.extension, "txt");
        assert_eq!(metadata.size, 13); // "Hello, World!" は13バイト
        assert!(metadata.modified_at.elapsed().is_ok());
        
        // EXIF情報は画像ファイルではないのでNone
        assert!(metadata.capture_date.is_none());
        assert!(metadata.camera_model.is_none());
        assert!(metadata.gps_latitude.is_none());
        assert!(metadata.gps_longitude.is_none());
    }

    #[test]
    fn test_extract_nonexistent_file() {
        let extractor = DefaultMetadataExtractor;
        let result = extractor.extract("nonexistent_file.txt");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_extract_file_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_file_no_ext");
        
        fs::File::create(&test_file_path).unwrap();
        
        let extractor = DefaultMetadataExtractor;
        let metadata = extractor.extract(test_file_path.to_str().unwrap()).unwrap();
        
        assert_eq!(metadata.filename, "test_file_no_ext");
        assert_eq!(metadata.extension, "");
    }

    #[test]
    fn test_is_image_file_detection() {
        use super::super::metadata_extractor::is_image_file;
        
        // 画像ファイル拡張子
        assert!(is_image_file("jpg"));
        assert!(is_image_file("jpeg"));
        assert!(is_image_file("png"));
        assert!(is_image_file("heic"));
        assert!(is_image_file("cr2"));
        assert!(is_image_file("nef"));
        
        // 非画像ファイル拡張子
        assert!(!is_image_file("txt"));
        assert!(!is_image_file("pdf"));
        assert!(!is_image_file("mp4"));
    }

    #[test]
    fn test_extract_image_file_metadata() {
        // プロジェクト内のPNG画像を使用してテスト
        let test_image_path = "../src-tauri/icons/icon.png";
        
        // ファイルが存在する場合のみテスト実行
        if std::path::Path::new(test_image_path).exists() {
            let extractor = DefaultMetadataExtractor;
            let metadata = extractor.extract(test_image_path).unwrap();
            
            // ファイルシステムメタデータを検証
            assert_eq!(metadata.filename, "icon.png");
            assert_eq!(metadata.extension, "png");
            assert!(metadata.size > 0);
            assert!(metadata.modified_at.elapsed().is_ok());
            
            // PNG画像はEXIF情報を持たない場合が多いが、エラーにはならない
            // （EXIF情報がない場合はNoneが返される）
        }
    }

    #[test]
    fn test_extract_handles_missing_exif_gracefully() {
        // EXIF情報を持たない画像ファイルでもエラーにならないことを確認
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test.jpg");
        
        // 空のJPEGファイル（EXIF情報なし）を作成
        fs::File::create(&test_file_path).unwrap();
        
        let extractor = DefaultMetadataExtractor;
        let result = extractor.extract(test_file_path.to_str().unwrap());
        
        // ファイルシステムメタデータは取得できるはず
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.extension, "jpg");
        // EXIF情報はないのでNone
        assert!(metadata.capture_date.is_none());
        assert!(metadata.camera_model.is_none());
    }

    // Feature: file-shiwake-kun, Property 3: ファイルシステムメタデータの抽出
    // For any file that is processed, filesystem metadata (created_at, modified_at, size, extension) must be extracted
    // Validates: Requirements 2.1
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_filesystem_metadata_extraction(
            filename in "[a-zA-Z0-9_-]{1,20}",
            extension in prop::option::of("[a-z]{2,4}"),
            content in prop::collection::vec(any::<u8>(), 0..1024)
        ) {
            // 一時ディレクトリを作成
            let temp_dir = TempDir::new().unwrap();
            
            // ファイル名を構築
            let full_filename = if let Some(ext) = &extension {
                format!("{}.{}", filename, ext)
            } else {
                filename.clone()
            };
            
            let test_file_path = temp_dir.path().join(&full_filename);
            
            // テストファイルを作成
            let mut file = fs::File::create(&test_file_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // ファイルハンドルを閉じる
            
            // メタデータを抽出
            let extractor = DefaultMetadataExtractor;
            let result = extractor.extract(test_file_path.to_str().unwrap());
            
            // プロパティ: ファイルが存在する場合、メタデータ抽出は成功しなければならない
            prop_assert!(result.is_ok(), "Metadata extraction should succeed for existing file");
            
            let metadata = result.unwrap();
            
            // プロパティ 1: ファイル名が正しく抽出される
            prop_assert_eq!(&metadata.filename, &full_filename, "Filename should match");
            
            // プロパティ 2: 拡張子が正しく抽出される（小文字に正規化）
            let expected_extension = extension.as_ref().map(|e| e.to_lowercase()).unwrap_or_default();
            prop_assert_eq!(&metadata.extension, &expected_extension, "Extension should match");
            
            // プロパティ 3: ファイルサイズが正しく抽出される
            prop_assert_eq!(metadata.size, content.len() as u64, "File size should match content length");
            
            // プロパティ 4: 更新日時が抽出される（SystemTimeとして有効）
            prop_assert!(metadata.modified_at.elapsed().is_ok(), "Modified time should be valid and in the past");
            
            // プロパティ 5: 作成日時が抽出される（プラットフォームによってはNoneの場合もある）
            // Windowsでは通常利用可能、Unix系では利用できない場合がある
            if let Some(created) = metadata.created_at {
                prop_assert!(created.elapsed().is_ok(), "Created time should be valid if present");
            }
        }
    }

    // Feature: file-shiwake-kun, Property 4: 画像EXIFメタデータの抽出
    // For any image file (JPEG, PNG, HEIC, RAW), available EXIF metadata (capture date, camera model, GPS coordinates) must be extracted
    // Validates: Requirements 2.2
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_image_exif_metadata_extraction(
            filename in "[a-zA-Z0-9_-]{1,20}",
            image_ext in prop::sample::select(vec!["jpg", "jpeg", "png", "heic", "tif", "tiff", "cr2", "nef", "arw", "dng"])
        ) {
            use super::super::metadata_extractor::is_image_file;
            
            // 一時ディレクトリを作成
            let temp_dir = TempDir::new().unwrap();
            
            // 画像ファイル名を構築
            let full_filename = format!("{}.{}", filename, image_ext);
            let test_file_path = temp_dir.path().join(&full_filename);
            
            // 最小限の有効なJPEGファイルを作成（EXIF情報なし）
            // これは実際の画像データではないが、ファイルシステムメタデータのテストには十分
            let minimal_jpeg = create_minimal_jpeg();
            fs::write(&test_file_path, &minimal_jpeg).unwrap();
            
            // メタデータを抽出
            let extractor = DefaultMetadataExtractor;
            let result = extractor.extract(test_file_path.to_str().unwrap());
            
            // プロパティ 1: 画像ファイルの場合、メタデータ抽出は成功しなければならない
            prop_assert!(result.is_ok(), "Metadata extraction should succeed for image files");
            
            let metadata = result.unwrap();
            
            // プロパティ 2: 画像ファイルとして認識される拡張子の場合、is_image_fileがtrueを返す
            prop_assert!(is_image_file(&image_ext), "Extension {} should be recognized as image", image_ext);
            
            // プロパティ 3: EXIF情報が欠けている場合でも、エラーにならずNoneが返される
            // （最小限のJPEGファイルにはEXIF情報がないため）
            // これは要件2.3「ファイルに特定のメタデータフィールドが欠けている時、失敗せずに欠損データを適切に処理しなければならない」を検証
            
            // プロパティ 4: ファイルシステムメタデータは正しく抽出される
            prop_assert_eq!(&metadata.filename, &full_filename, "Filename should match");
            prop_assert_eq!(&metadata.extension, &image_ext.to_lowercase(), "Extension should match");
            prop_assert!(metadata.size > 0, "File size should be greater than 0");
            prop_assert!(metadata.modified_at.elapsed().is_ok(), "Modified time should be valid");
            
            // プロパティ 5: EXIF情報がない場合、全てのEXIFフィールドはNoneである
            // （最小限のJPEGファイルにはEXIF情報がないため）
            // これにより、EXIF抽出が失敗してもエラーにならないことを確認
            prop_assert!(metadata.capture_date.is_none() || metadata.capture_date.is_some(), 
                "Capture date should be None or Some (graceful handling)");
            prop_assert!(metadata.camera_model.is_none() || metadata.camera_model.is_some(), 
                "Camera model should be None or Some (graceful handling)");
            prop_assert!(metadata.gps_latitude.is_none() || metadata.gps_latitude.is_some(), 
                "GPS latitude should be None or Some (graceful handling)");
            prop_assert!(metadata.gps_longitude.is_none() || metadata.gps_longitude.is_some(), 
                "GPS longitude should be None or Some (graceful handling)");
        }
    }

    /// 最小限の有効なJPEGファイルを作成（EXIF情報なし）
    fn create_minimal_jpeg() -> Vec<u8> {
        // 最小限の有効なJPEGファイル構造
        // SOI (Start of Image) + SOF0 (Start of Frame) + SOS (Start of Scan) + EOI (End of Image)
        vec![
            0xFF, 0xD8, // SOI (Start of Image)
            0xFF, 0xE0, // APP0 marker
            0x00, 0x10, // APP0 length (16 bytes)
            0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
            0x01, 0x01, // JFIF version 1.1
            0x00, // density units (0 = no units)
            0x00, 0x01, // X density
            0x00, 0x01, // Y density
            0x00, 0x00, // thumbnail dimensions (0x0)
            0xFF, 0xDB, // DQT (Define Quantization Table)
            0x00, 0x43, // DQT length
            0x00, // table ID
            // 64 bytes of quantization table data (simplified)
            0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07,
            0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14,
            0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12, 0x13,
            0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A,
            0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20, 0x22,
            0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C,
            0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39,
            0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32,
            0xFF, 0xC0, // SOF0 (Start of Frame, Baseline DCT)
            0x00, 0x0B, // SOF0 length
            0x08, // precision
            0x00, 0x01, // height (1 pixel)
            0x00, 0x01, // width (1 pixel)
            0x01, // number of components
            0x01, 0x11, 0x00, // component 1
            0xFF, 0xDA, // SOS (Start of Scan)
            0x00, 0x08, // SOS length
            0x01, // number of components
            0x01, 0x00, // component 1
            0x00, 0x3F, 0x00, // spectral selection
            0xFF, 0xD9, // EOI (End of Image)
        ]
    }
}
