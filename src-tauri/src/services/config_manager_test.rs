// Feature: file-shiwake-kun, Property 10: 設定の永続化（ラウンドトリップ）

#[cfg(test)]
mod tests {
    use super::super::config_manager::ConfigManager;
    use crate::models::{Config, Rule, Condition, OperationType};
    use proptest::prelude::*;
    use tempfile::TempDir;
    use std::collections::HashSet;

    // Strategy for generating OperationType
    fn operation_type_strategy() -> impl Strategy<Value = OperationType> {
        prop_oneof![
            Just(OperationType::Move),
            Just(OperationType::Copy),
        ]
    }

    // Strategy for generating valid Condition based on operator
    fn condition_strategy() -> impl Strategy<Value = Condition> {
        // Generate valid operators
        let operator_strategy = prop_oneof![
            Just("==".to_string()),
            Just("!=".to_string()),
            Just("in".to_string()),
            Just("not_in".to_string()),
            Just("exists".to_string()),
            Just(">".to_string()),
            Just("<".to_string()),
            Just(">=".to_string()),
            Just("<=".to_string()),
        ];

        (
            "[a-z_]{1,20}",  // field name
            operator_strategy,
        ).prop_flat_map(|(field, operator)| {
            // Generate appropriate value based on operator
            let value_strategy = match operator.as_ref() {
                "exists" => {
                    // exists operator - use boolean or empty string (TOML doesn't support null)
                    prop_oneof![
                        any::<bool>().prop_map(|b| serde_json::json!(b)),
                        Just(serde_json::json!("")),
                    ].boxed()
                }
                "in" | "not_in" => {
                    // in/not_in require arrays
                    prop::collection::vec(
                        prop_oneof![
                            any::<String>().prop_map(|s| serde_json::json!(s)),
                            any::<i64>().prop_map(|i| serde_json::json!(i)),
                        ],
                        1..5
                    ).prop_map(|v| serde_json::json!(v)).boxed()
                }
                _ => {
                    // Other operators need non-null values
                    prop_oneof![
                        "[a-z0-9._-]{1,20}".prop_map(|s| serde_json::json!(s)),
                        any::<i64>().prop_map(|i| serde_json::json!(i)),
                        any::<bool>().prop_map(|b| serde_json::json!(b)),
                    ].boxed()
                }
            };

            (Just(field), Just(operator), value_strategy)
        }).prop_map(|(field, operator, value)| Condition {
            field,
            operator,
            value,
        })
    }

    // Strategy for generating valid path patterns (no invalid chars, no ..)
    fn path_pattern_strategy() -> impl Strategy<Value = String> {
        // Generate paths without invalid characters
        prop::collection::vec("[A-Za-z0-9_-]{1,10}", 1..4)
            .prop_map(|parts| {
                let mut path = parts.join("/");
                // Add some template variables
                if path.len() < 50 {
                    path.push_str("/{year}");
                }
                path
            })
    }

    // Strategy for generating Rule with unique IDs
    fn rule_strategy() -> impl Strategy<Value = Rule> {
        (
            "[a-z0-9-]{5,20}",  // id - ensure minimum length for uniqueness
            "[A-Za-z0-9 ]{5,50}",  // name - ensure not empty
            any::<i32>(),
            prop::collection::vec(condition_strategy(), 1..5),  // at least 1 condition
            path_pattern_strategy(),
            operation_type_strategy(),
        ).prop_map(|(id, name, priority, conditions, destination_pattern, operation)| Rule {
            id,
            name,
            priority,
            conditions,
            destination_pattern,
            operation,
        })
    }

    // Strategy for generating Config with unique rule IDs
    fn config_strategy() -> impl Strategy<Value = Config> {
        (
            prop::collection::vec(rule_strategy(), 0..10),
            "[A-Za-z0-9/_]{5,100}",  // default_destination - ensure not empty
            any::<bool>(),
            "[A-Za-z0-9._-]{5,50}",  // log_path - ensure not empty
        ).prop_filter_map(
            "Config must have unique rule IDs",
            |(rules, default_destination, preview_mode, log_path)| {
                // Ensure rule IDs are unique
                let mut seen_ids = HashSet::new();
                let mut unique_rules = Vec::new();
                
                for rule in rules {
                    if seen_ids.insert(rule.id.clone()) {
                        unique_rules.push(rule);
                    }
                }
                
                Some(Config {
                    rules: unique_rules,
                    default_destination,
                    preview_mode,
                    log_path,
                })
            }
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_config_roundtrip(config in config_strategy()) {
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.toml");
            
            // Create ConfigManager
            let manager = ConfigManager::new(config_path);
            
            // Save the config
            manager.save(&config).expect("Failed to save config");
            
            // Load the config back
            let loaded_config = manager.load().expect("Failed to load config");
            
            // Assert that the loaded config equals the original
            prop_assert_eq!(config, loaded_config);
        }
    }

    // Unit tests for validation logic
    #[test]
    fn test_default_config_is_valid() {
        let config = ConfigManager::create_default_config();
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        assert!(manager.validate(&config).is_ok());
    }

    #[test]
    fn test_empty_default_destination_is_invalid() {
        let mut config = ConfigManager::create_default_config();
        config.default_destination = "".to_string();
        
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        assert!(manager.validate(&config).is_err());
    }

    #[test]
    fn test_duplicate_rule_ids_are_invalid() {
        let mut config = ConfigManager::create_default_config();
        let first_rule = config.rules[0].clone();
        config.rules.push(first_rule);
        
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        let result = manager.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate rule ID"));
    }

    #[test]
    fn test_path_traversal_is_invalid() {
        let mut config = ConfigManager::create_default_config();
        config.rules[0].destination_pattern = "../etc/passwd".to_string();
        
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        let result = manager.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path traversal"));
    }

    #[test]
    fn test_invalid_operator_is_rejected() {
        let mut config = ConfigManager::create_default_config();
        config.rules[0].conditions[0].operator = "invalid_op".to_string();
        
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        let result = manager.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid operator"));
    }

    #[test]
    fn test_rule_without_conditions_is_invalid() {
        let mut config = ConfigManager::create_default_config();
        config.rules[0].conditions.clear();
        
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().join("config.toml"));
        
        let result = manager.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have at least one condition"));
    }

    #[test]
    fn test_load_creates_default_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        // Ensure file doesn't exist
        assert!(!config_path.exists());
        
        let manager = ConfigManager::new(config_path.clone());
        let config = manager.load().expect("Failed to load config");
        
        // File should now exist
        assert!(config_path.exists());
        
        // Config should be valid
        assert!(manager.validate(&config).is_ok());
    }

    // Feature: file-shiwake-kun, Property 11: 設定の検証
    // Validates: Requirements 5.5
    
    // Strategy for generating invalid configurations
    fn invalid_config_strategy() -> impl Strategy<Value = Config> {
        prop_oneof![
            // Invalid: empty default_destination
            config_strategy().prop_map(|mut c| {
                c.default_destination = "".to_string();
                c
            }),
            // Invalid: empty log_path
            config_strategy().prop_map(|mut c| {
                c.log_path = "".to_string();
                c
            }),
            // Invalid: whitespace-only default_destination
            config_strategy().prop_map(|mut c| {
                c.default_destination = "   ".to_string();
                c
            }),
            // Invalid: whitespace-only log_path
            config_strategy().prop_map(|mut c| {
                c.log_path = "   ".to_string();
                c
            }),
            // Invalid: duplicate rule IDs
            config_strategy().prop_filter_map(
                "Need at least 2 rules",
                |mut c| {
                    if c.rules.len() >= 2 {
                        c.rules[1].id = c.rules[0].id.clone();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: empty rule ID
            config_strategy().prop_filter_map(
                "Need at least 1 rule",
                |mut c| {
                    if !c.rules.is_empty() {
                        c.rules[0].id = "".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: empty rule name
            config_strategy().prop_filter_map(
                "Need at least 1 rule",
                |mut c| {
                    if !c.rules.is_empty() {
                        c.rules[0].name = "".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: empty destination pattern
            config_strategy().prop_filter_map(
                "Need at least 1 rule",
                |mut c| {
                    if !c.rules.is_empty() {
                        c.rules[0].destination_pattern = "".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: path traversal in destination pattern
            config_strategy().prop_filter_map(
                "Need at least 1 rule",
                |mut c| {
                    if !c.rules.is_empty() {
                        c.rules[0].destination_pattern = "../etc/passwd".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: rule without conditions
            config_strategy().prop_filter_map(
                "Need at least 1 rule",
                |mut c| {
                    if !c.rules.is_empty() {
                        c.rules[0].conditions.clear();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: empty condition field
            config_strategy().prop_filter_map(
                "Need at least 1 rule with conditions",
                |mut c| {
                    if !c.rules.is_empty() && !c.rules[0].conditions.is_empty() {
                        c.rules[0].conditions[0].field = "".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: empty condition operator
            config_strategy().prop_filter_map(
                "Need at least 1 rule with conditions",
                |mut c| {
                    if !c.rules.is_empty() && !c.rules[0].conditions.is_empty() {
                        c.rules[0].conditions[0].operator = "".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: unsupported operator
            config_strategy().prop_filter_map(
                "Need at least 1 rule with conditions",
                |mut c| {
                    if !c.rules.is_empty() && !c.rules[0].conditions.is_empty() {
                        c.rules[0].conditions[0].operator = "invalid_operator".to_string();
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: 'in' operator with non-array value
            config_strategy().prop_filter_map(
                "Need at least 1 rule with conditions",
                |mut c| {
                    if !c.rules.is_empty() && !c.rules[0].conditions.is_empty() {
                        c.rules[0].conditions[0].operator = "in".to_string();
                        c.rules[0].conditions[0].value = serde_json::json!("not_an_array");
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
            // Invalid: '==' operator with null value
            config_strategy().prop_filter_map(
                "Need at least 1 rule with conditions",
                |mut c| {
                    if !c.rules.is_empty() && !c.rules[0].conditions.is_empty() {
                        c.rules[0].conditions[0].operator = "==".to_string();
                        c.rules[0].conditions[0].value = serde_json::Value::Null;
                        Some(c)
                    } else {
                        None
                    }
                }
            ),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_invalid_configs_are_rejected(invalid_config in invalid_config_strategy()) {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.toml");
            let manager = ConfigManager::new(config_path);
            
            // Validate should reject the invalid config
            let result = manager.validate(&invalid_config);
            prop_assert!(result.is_err(), "Expected validation to fail for invalid config, but it passed");
        }
    }
}
