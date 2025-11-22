use super::*;
use crate::models::{Rule, Condition, OperationType, FileMetadata};
use std::time::SystemTime;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> FileMetadata {
        FileMetadata {
            filename: "test_photo.jpg".to_string(),
            extension: "jpg".to_string(),
            size: 1024 * 1024 * 5, // 5MB
            created_at: Some(SystemTime::now()),
            modified_at: SystemTime::now(),
            capture_date: Some(SystemTime::now()),
            camera_model: Some("Canon EOS 5D".to_string()),
            gps_latitude: Some(35.6762),
            gps_longitude: Some(139.6503),
        }
    }

    fn create_test_rule(priority: i32) -> Rule {
        Rule {
            id: format!("rule-{}", priority),
            name: "Test Rule".to_string(),
            priority,
            conditions: vec![
                Condition {
                    field: "extension".to_string(),
                    operator: "in".to_string(),
                    value: serde_json::json!([".jpg", ".jpeg", ".png"]),
                },
                Condition {
                    field: "capture_date".to_string(),
                    operator: "exists".to_string(),
                    value: serde_json::json!(null),
                },
            ],
            destination_pattern: "Photos/{year}/{month}".to_string(),
            operation: OperationType::Move,
        }
    }

    #[test]
    fn test_rule_matching_with_extension() {
        let rule = create_test_rule(1);
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_some());
    }

    #[test]
    fn test_rule_not_matching_wrong_extension() {
        let mut rule = create_test_rule(1);
        rule.conditions[0].value = serde_json::json!([".pdf", ".doc"]);
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_none());
    }

    #[test]
    fn test_priority_ordering() {
        let rule1 = create_test_rule(1);
        let rule2 = create_test_rule(10);
        let rule3 = create_test_rule(5);
        
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule1, rule2.clone(), rule3]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, rule2.id); // Should match highest priority
    }

    #[test]
    fn test_template_expansion() {
        let rule = create_test_rule(1);
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule.clone()]);

        let result = engine.apply_rule(&rule, &metadata);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.contains("Photos/"));
        assert!(path.contains("/")); // Should have year/month structure
    }

    #[test]
    fn test_template_with_camera_model() {
        let mut rule = create_test_rule(1);
        rule.destination_pattern = "Photos/{camera}/{year}".to_string();
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule.clone()]);

        let result = engine.apply_rule(&rule, &metadata);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.contains("Canon")); // Camera model should be included
    }

    #[test]
    fn test_template_with_filename() {
        let mut rule = create_test_rule(1);
        rule.destination_pattern = "Photos/{filename}.{extension}".to_string();
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule.clone()]);

        let result = engine.apply_rule(&rule, &metadata);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.contains("test_photo"));
        assert!(path.contains(".jpg"));
    }

    #[test]
    fn test_size_condition() {
        let mut rule = create_test_rule(1);
        rule.conditions = vec![
            Condition {
                field: "size".to_string(),
                operator: ">".to_string(),
                value: serde_json::json!(1024 * 1024), // 1MB
            },
        ];
        
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_some());
    }

    #[test]
    fn test_camera_model_condition() {
        let mut rule = create_test_rule(1);
        rule.conditions = vec![
            Condition {
                field: "camera_model".to_string(),
                operator: "contains".to_string(),
                value: serde_json::json!("Canon"),
            },
        ];
        
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_some());
    }

    #[test]
    fn test_no_matching_rule() {
        let mut rule = create_test_rule(1);
        rule.conditions = vec![
            Condition {
                field: "extension".to_string(),
                operator: "==".to_string(),
                value: serde_json::json!(".pdf"),
            },
        ];
        
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_none());
    }

    #[test]
    fn test_gps_exists_condition() {
        let mut rule = create_test_rule(1);
        rule.conditions = vec![
            Condition {
                field: "gps_latitude".to_string(),
                operator: "exists".to_string(),
                value: serde_json::json!(null),
            },
        ];
        
        let metadata = create_test_metadata();
        let engine = RuleEngine::new(vec![rule]);

        let matched = engine.find_matching_rule(&metadata);
        assert!(matched.is_some());
    }

    #[test]
    fn test_sanitize_camera_model_in_path() {
        let mut rule = create_test_rule(1);
        rule.destination_pattern = "{camera}".to_string();
        
        let mut metadata = create_test_metadata();
        metadata.camera_model = Some("Canon/EOS:5D*Mark?IV".to_string());
        
        let engine = RuleEngine::new(vec![rule.clone()]);
        let result = engine.apply_rule(&rule, &metadata);
        
        assert!(result.is_ok());
        let path = result.unwrap();
        // Should not contain invalid path characters in the camera model part
        assert!(!path.contains('/'));
        assert!(!path.contains(':'));
        assert!(!path.contains('*'));
        assert!(!path.contains('?'));
        assert!(path.contains('_')); // Invalid chars should be replaced with underscore
    }
}

// Feature: file-shiwake-kun, Property 5: テンプレート変数の展開
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::time::{SystemTime, Duration};

    // Strategy for generating valid file extensions
    fn extension_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "jpg".to_string(),
            "jpeg".to_string(),
            "png".to_string(),
            "pdf".to_string(),
            "txt".to_string(),
            "doc".to_string(),
            "mp4".to_string(),
            "heic".to_string(),
        ])
    }

    // Strategy for generating valid filenames
    fn filename_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{1,20}\\.[a-z]{3,4}"
            .prop_map(|s| s.to_string())
    }

    // Strategy for generating camera models
    fn camera_model_strategy() -> impl Strategy<Value = Option<String>> {
        prop::option::of(prop::sample::select(vec![
            "Canon EOS 5D".to_string(),
            "Nikon D850".to_string(),
            "Sony A7III".to_string(),
            "iPhone 12 Pro".to_string(),
            "Samsung Galaxy S21".to_string(),
        ]))
    }

    // Strategy for generating SystemTime
    fn system_time_strategy() -> impl Strategy<Value = SystemTime> {
        // Generate times within a reasonable range (last 10 years)
        (0u64..315360000u64).prop_map(|secs| {
            SystemTime::UNIX_EPOCH + Duration::from_secs(1609459200 + secs) // Starting from 2021-01-01
        })
    }

    // Strategy for generating FileMetadata
    fn metadata_strategy() -> impl Strategy<Value = FileMetadata> {
        (
            filename_strategy(),
            extension_strategy(),
            1u64..1024 * 1024 * 1024, // 1 byte to 1GB
            prop::option::of(system_time_strategy()),
            system_time_strategy(),
            prop::option::of(system_time_strategy()),
            camera_model_strategy(),
            prop::option::of(-90.0f64..90.0f64),
            prop::option::of(-180.0f64..180.0f64),
        ).prop_map(|(filename, extension, size, created_at, modified_at, capture_date, camera_model, gps_lat, gps_lon)| {
            FileMetadata {
                filename,
                extension,
                size,
                created_at,
                modified_at,
                capture_date,
                camera_model,
                gps_latitude: gps_lat,
                gps_longitude: gps_lon,
            }
        })
    }

    // Strategy for generating destination patterns with template variables
    fn destination_pattern_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "Photos/{year}/{month}".to_string(),
            "Backup/{year}/{month}/{day}".to_string(),
            "{camera}/{year}".to_string(),
            "Files/{extension}/{year}".to_string(),
            "{year}/{month}/{filename}.{extension}".to_string(),
            "Archive/{year}/{size_mb}MB".to_string(),
            "{camera}/{year}/{month}/{day}".to_string(),
            "Organized/{extension}/{year}/{month}".to_string(),
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_template_variable_expansion(
            metadata in metadata_strategy(),
            pattern in destination_pattern_strategy()
        ) {
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: pattern.clone(),
                operation: OperationType::Move,
            };

            let engine = RuleEngine::new(vec![rule.clone()]);
            let result = engine.apply_rule(&rule, &metadata);

            // The expansion should always succeed
            prop_assert!(result.is_ok(), "Template expansion failed: {:?}", result);

            let expanded = result.unwrap();

            // Verify that template variables are expanded correctly
            
            // 1. {year} should be replaced with a 4-digit year
            if pattern.contains("{year}") {
                prop_assert!(!expanded.contains("{year}"), "{{year}} was not expanded in: {}", expanded);
                // Should contain a 4-digit year (2021-2031 based on our time strategy)
                prop_assert!(
                    expanded.contains("202") || expanded.contains("203"),
                    "Expanded path should contain a valid year: {}", expanded
                );
            }

            // 2. {month} should be replaced with a 2-digit month (01-12)
            if pattern.contains("{month}") {
                prop_assert!(!expanded.contains("{month}"), "{{month}} was not expanded in: {}", expanded);
            }

            // 3. {day} should be replaced with a 2-digit day (01-31)
            if pattern.contains("{day}") {
                prop_assert!(!expanded.contains("{day}"), "{{day}} was not expanded in: {}", expanded);
            }

            // 4. {extension} should be replaced with the file extension
            if pattern.contains("{extension}") {
                prop_assert!(!expanded.contains("{extension}"), "{{extension}} was not expanded in: {}", expanded);
                prop_assert!(
                    expanded.contains(&metadata.extension),
                    "Expanded path should contain extension '{}': {}", metadata.extension, expanded
                );
            }

            // 5. {camera} should be replaced with camera model (if present in metadata)
            if pattern.contains("{camera}") {
                prop_assert!(!expanded.contains("{camera}"), "{{camera}} was not expanded in: {}", expanded);
                if metadata.camera_model.is_some() {
                    // Camera model should be sanitized (no invalid path characters)
                    prop_assert!(
                        !expanded.contains('/') || expanded.split('/').any(|part| !part.is_empty()),
                        "Expanded path should not contain invalid path separators: {}", expanded
                    );
                }
            }

            // 6. {filename} should be replaced with the filename (without extension)
            if pattern.contains("{filename}") {
                prop_assert!(!expanded.contains("{filename}"), "{{filename}} was not expanded in: {}", expanded);
            }

            // 7. {size_mb} should be replaced with file size in MB
            if pattern.contains("{size_mb}") {
                prop_assert!(!expanded.contains("{size_mb}"), "{{size_mb}} was not expanded in: {}", expanded);
            }

            // 8. The expanded path should not contain any unreplaced template variables
            prop_assert!(
                !expanded.contains('{') || !expanded.contains('}'),
                "Expanded path contains unreplaced template variables: {}", expanded
            );
        }

        // Feature: file-shiwake-kun, Property 6: ルール優先順位の遵守
        #[test]
        fn test_rule_priority_adherence(
            metadata in metadata_strategy(),
            priorities in prop::collection::vec(any::<i32>(), 2..10)
        ) {
            // Create multiple rules that all match the same metadata
            // Each rule has a different priority
            let rules: Vec<Rule> = priorities.iter().enumerate().map(|(idx, &priority)| {
                Rule {
                    id: format!("rule-{}", idx),
                    name: format!("Rule {}", idx),
                    priority,
                    // Conditions that will match any file (always true)
                    conditions: vec![
                        Condition {
                            field: "modified_at".to_string(),
                            operator: "exists".to_string(),
                            value: serde_json::json!(null),
                        },
                    ],
                    destination_pattern: format!("dest-{}/{{}}", idx),
                    operation: OperationType::Move,
                }
            }).collect();

            // Find the highest priority value
            let max_priority = priorities.iter().max().copied().unwrap();
            
            // Find the index of the rule with the highest priority
            let expected_rule_idx = priorities.iter().position(|&p| p == max_priority).unwrap();

            // Create the rule engine (it should sort by priority)
            let engine = RuleEngine::new(rules.clone());

            // Find the matching rule
            let matched_rule = engine.find_matching_rule(&metadata);

            // The matched rule should be the one with the highest priority
            prop_assert!(
                matched_rule.is_some(),
                "A rule should match since all rules have always-true conditions"
            );

            let matched = matched_rule.unwrap();
            prop_assert_eq!(
                matched.priority,
                max_priority,
                "The matched rule should have the highest priority. Expected priority: {}, Got: {}",
                max_priority,
                matched.priority
            );

            // Verify it's the correct rule by ID
            prop_assert_eq!(
                &matched.id,
                &format!("rule-{}", expected_rule_idx),
                "The matched rule should be the one with the highest priority"
            );
        }
    }
}
