#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json;

    #[test]
    fn test_rule_json_serialization() {
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![
                Condition {
                    field: "extension".to_string(),
                    operator: "==".to_string(),
                    value: serde_json::json!(".jpg"),
                }
            ],
            destination_pattern: "/photos/{year}".to_string(),
            operation: OperationType::Move,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: Rule = serde_json::from_str(&json).unwrap();
        
        assert_eq!(rule.id, deserialized.id);
        assert_eq!(rule.name, deserialized.name);
        assert_eq!(rule.priority, deserialized.priority);
    }

    #[test]
    fn test_config_toml_serialization() {
        let config = Config {
            rules: vec![],
            default_destination: "/default".to_string(),
            preview_mode: false,
            log_path: "app.log".to_string(),
        };

        // Test TOML serialization
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.default_destination, deserialized.default_destination);
        assert_eq!(config.preview_mode, deserialized.preview_mode);
        assert_eq!(config.log_path, deserialized.log_path);
    }

    #[test]
    fn test_file_metadata_json_serialization() {
        use std::time::SystemTime;
        
        let metadata = FileMetadata {
            filename: "test.jpg".to_string(),
            extension: "jpg".to_string(),
            size: 1024,
            created_at: Some(SystemTime::now()),
            modified_at: SystemTime::now(),
            capture_date: None,
            camera_model: Some("Canon EOS".to_string()),
            gps_latitude: Some(35.6762),
            gps_longitude: Some(139.6503),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: FileMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.filename, deserialized.filename);
        assert_eq!(metadata.extension, deserialized.extension);
        assert_eq!(metadata.size, deserialized.size);
    }

    #[test]
    fn test_process_result_json_serialization() {
        let result = ProcessResult {
            source_path: "/source/file.jpg".to_string(),
            destination_path: Some("/dest/file.jpg".to_string()),
            success: true,
            error_message: None,
            matched_rule: Some("rule-001".to_string()),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ProcessResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(result.source_path, deserialized.source_path);
        assert_eq!(result.success, deserialized.success);
    }

    #[test]
    fn test_conflict_resolution_serialization() {
        let resolution = ConflictResolution::Overwrite;
        
        // Test JSON serialization
        let json = serde_json::to_string(&resolution).unwrap();
        let deserialized: ConflictResolution = serde_json::from_str(&json).unwrap();
        
        assert_eq!(resolution, deserialized);
    }
}
