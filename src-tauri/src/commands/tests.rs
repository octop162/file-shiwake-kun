#[cfg(test)]
mod tests {
    use crate::commands::{load_config, save_config, get_file_info, process_files};
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_load_config_creates_default() {
        // This test verifies that load_config creates a default config if none exists
        let result = load_config().await;
        assert!(result.is_ok(), "load_config should succeed");
        
        let config = result.unwrap();
        assert!(!config.rules.is_empty(), "Default config should have rules");
        assert!(!config.default_destination.is_empty(), "Default destination should be set");
    }

    #[tokio::test]
    async fn test_save_and_load_config_roundtrip() {
        // First load the current config
        let original_config = load_config().await.unwrap();
        
        // Create a modified config
        let mut test_config = original_config.clone();
        test_config.preview_mode = !test_config.preview_mode; // Toggle preview mode
        test_config.log_path = "test_modified.log".to_string();

        // Save the modified config
        let save_result = save_config(test_config.clone()).await;
        assert!(save_result.is_ok(), "save_config should succeed");

        // Load the config back
        let load_result = load_config().await;
        assert!(load_result.is_ok(), "load_config should succeed");

        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.preview_mode, test_config.preview_mode);
        assert_eq!(loaded_config.log_path, test_config.log_path);
        
        // Restore original config
        save_config(original_config).await.ok();
    }

    #[tokio::test]
    async fn test_get_file_info_for_existing_file() {
        // Create a temporary file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // Get file info
        let result = get_file_info(file_path.to_string_lossy().to_string()).await;
        assert!(result.is_ok(), "get_file_info should succeed for existing file");

        let file_info = result.unwrap();
        assert_eq!(file_info.name, "test.txt");
        assert!(file_info.size > 0);
    }

    #[tokio::test]
    async fn test_get_file_info_for_nonexistent_file() {
        // Try to get info for a file that doesn't exist
        let result = get_file_info("/nonexistent/file.txt".to_string()).await;
        assert!(result.is_err(), "get_file_info should fail for nonexistent file");
    }

    #[tokio::test]
    async fn test_process_files_with_empty_list() {
        // Process an empty list of files
        let result = process_files(vec![]).await;
        assert!(result.is_ok(), "process_files should succeed with empty list");
        
        let results = result.unwrap();
        assert_eq!(results.len(), 0, "Should return empty results for empty input");
    }
}
