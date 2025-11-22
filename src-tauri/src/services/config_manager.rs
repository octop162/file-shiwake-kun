use crate::models::{Config, Rule, Condition, OperationType};
use std::path::PathBuf;
use std::fs;
use std::collections::HashSet;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Load configuration from TOML file
    /// If file doesn't exist, creates a default configuration
    /// Requirement 5.1: Load config from TOML file on startup
    /// Requirement 5.3: Create default config if file doesn't exist
    pub fn load(&self) -> Result<Config, String> {
        // Check if config file exists
        if !self.config_path.exists() {
            // Create default config
            let default_config = Self::create_default_config();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        // Requirement 5.4: Handle invalid TOML syntax
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}. Using default configuration.", e))?;
        
        // Validate the loaded config
        self.validate(&config)?;
        
        Ok(config)
    }

    /// Save configuration to TOML file
    /// Requirement 5.2: Save changes to TOML file
    pub fn save(&self, config: &Config) -> Result<(), String> {
        // Validate before saving
        self.validate(config)?;
        
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        fs::write(&self.config_path, toml_string)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// Create a default configuration
    /// Requirement 5.3: Create default config file if it doesn't exist
    pub fn create_default_config() -> Config {
        Config {
            rules: vec![
                Rule {
                    id: "default-images".to_string(),
                    name: "画像を年月別に整理".to_string(),
                    priority: 1,
                    conditions: vec![
                        Condition {
                            field: "extension".to_string(),
                            operator: "in".to_string(),
                            value: serde_json::json!([".jpg", ".jpeg", ".png", ".heic", ".raw"]),
                        },
                    ],
                    destination_pattern: "Pictures/{year}/{month}".to_string(),
                    operation: OperationType::Move,
                },
                Rule {
                    id: "default-documents".to_string(),
                    name: "ドキュメントを整理".to_string(),
                    priority: 2,
                    conditions: vec![
                        Condition {
                            field: "extension".to_string(),
                            operator: "in".to_string(),
                            value: serde_json::json!([".pdf", ".doc", ".docx", ".txt"]),
                        },
                    ],
                    destination_pattern: "Documents/{year}".to_string(),
                    operation: OperationType::Copy,
                },
            ],
            default_destination: "Unsorted".to_string(),
            preview_mode: false,
            log_path: "file-shiwake-kun.log".to_string(),
        }
    }

    /// Validate configuration
    /// Requirement 5.5: Validate config values and reject invalid rules/paths
    pub fn validate(&self, config: &Config) -> Result<(), String> {
        // Validate default_destination is not empty
        if config.default_destination.trim().is_empty() {
            return Err("default_destination cannot be empty".to_string());
        }

        // Validate log_path is not empty
        if config.log_path.trim().is_empty() {
            return Err("log_path cannot be empty".to_string());
        }

        // Validate rules
        let mut rule_ids = HashSet::new();
        
        for rule in &config.rules {
            // Check for duplicate rule IDs
            if !rule_ids.insert(&rule.id) {
                return Err(format!("Duplicate rule ID found: {}", rule.id));
            }

            // Validate rule ID is not empty
            if rule.id.trim().is_empty() {
                return Err("Rule ID cannot be empty".to_string());
            }

            // Validate rule name is not empty
            if rule.name.trim().is_empty() {
                return Err(format!("Rule name cannot be empty for rule ID: {}", rule.id));
            }

            // Validate destination pattern is not empty
            if rule.destination_pattern.trim().is_empty() {
                return Err(format!("Destination pattern cannot be empty for rule: {}", rule.name));
            }

            // Validate destination pattern doesn't contain invalid characters
            Self::validate_path_pattern(&rule.destination_pattern)?;

            // Validate conditions
            if rule.conditions.is_empty() {
                return Err(format!("Rule '{}' must have at least one condition", rule.name));
            }

            for condition in &rule.conditions {
                Self::validate_condition(condition)?;
            }
        }

        Ok(())
    }

    /// Validate a path pattern for invalid characters
    fn validate_path_pattern(pattern: &str) -> Result<(), String> {
        // Check for invalid characters in path (platform-specific)
        let invalid_chars = if cfg!(windows) {
            vec!['<', '>', ':', '"', '|', '?', '*']
        } else {
            vec!['\0']
        };

        for ch in invalid_chars {
            if pattern.contains(ch) {
                return Err(format!("Invalid character '{}' in path pattern: {}", ch, pattern));
            }
        }

        // Check for path traversal attempts
        if pattern.contains("..") {
            return Err(format!("Path traversal '..' not allowed in pattern: {}", pattern));
        }

        Ok(())
    }

    /// Validate a condition
    fn validate_condition(condition: &Condition) -> Result<(), String> {
        // Validate field is not empty
        if condition.field.trim().is_empty() {
            return Err("Condition field cannot be empty".to_string());
        }

        // Validate operator is not empty
        if condition.operator.trim().is_empty() {
            return Err(format!("Condition operator cannot be empty for field: {}", condition.field));
        }

        // Validate operator is one of the supported operators
        let valid_operators = vec!["==", "!=", "in", "not_in", "exists", ">", "<", ">=", "<="];
        if !valid_operators.contains(&condition.operator.as_str()) {
            return Err(format!(
                "Invalid operator '{}' for field '{}'. Valid operators: {}",
                condition.operator,
                condition.field,
                valid_operators.join(", ")
            ));
        }

        // Validate value based on operator
        match condition.operator.as_str() {
            "exists" => {
                // exists operator can have boolean, empty string, or null value
                // (TOML doesn't support null, so we accept empty string as equivalent)
                if !condition.value.is_null() 
                    && !condition.value.is_boolean() 
                    && !(condition.value.is_string() && condition.value.as_str().unwrap_or("") == "") {
                    return Err(format!(
                        "Operator 'exists' should have boolean, empty string, or no value for field: {}",
                        condition.field
                    ));
                }
            }
            "in" | "not_in" => {
                // in/not_in operators require array values
                if !condition.value.is_array() {
                    return Err(format!(
                        "Operator '{}' requires an array value for field: {}",
                        condition.operator, condition.field
                    ));
                }
            }
            _ => {
                // Other operators require non-null values
                if condition.value.is_null() {
                    return Err(format!(
                        "Operator '{}' requires a value for field: {}",
                        condition.operator, condition.field
                    ));
                }
            }
        }

        Ok(())
    }
}
