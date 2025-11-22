use crate::models::{Rule, Condition, FileMetadata};
use std::path::Path;

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Rule>) -> Self {
        // Sort rules by priority (higher priority first)
        let mut sorted_rules = rules;
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Self { rules: sorted_rules }
    }

    /// Find the first matching rule based on priority
    pub fn find_matching_rule(&self, metadata: &FileMetadata) -> Option<&Rule> {
        self.rules.iter().find(|rule| self.matches_rule(rule, metadata))
    }

    /// Check if a rule matches the given metadata
    fn matches_rule(&self, rule: &Rule, metadata: &FileMetadata) -> bool {
        // All conditions must be satisfied
        rule.conditions.iter().all(|condition| self.matches_condition(condition, metadata))
    }

    /// Check if a single condition matches the metadata
    fn matches_condition(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.field.as_str() {
            "extension" => self.match_extension(condition, metadata),
            "size" => self.match_size(condition, metadata),
            "created_at" => self.match_created_at(condition, metadata),
            "modified_at" => self.match_modified_at(condition, metadata),
            "capture_date" => self.match_capture_date(condition, metadata),
            "camera_model" => self.match_camera_model(condition, metadata),
            "gps_latitude" => self.match_gps_latitude(condition, metadata),
            "gps_longitude" => self.match_gps_longitude(condition, metadata),
            "filename" => self.match_filename(condition, metadata),
            _ => false, // Unknown field
        }
    }

    fn match_extension(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "==" => {
                if let Some(value) = condition.value.as_str() {
                    metadata.extension.eq_ignore_ascii_case(value.trim_start_matches('.'))
                } else {
                    false
                }
            }
            "in" => {
                if let Some(values) = condition.value.as_array() {
                    values.iter().any(|v| {
                        if let Some(ext) = v.as_str() {
                            metadata.extension.eq_ignore_ascii_case(ext.trim_start_matches('.'))
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn match_size(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        if let Some(value) = condition.value.as_u64() {
            match condition.operator.as_str() {
                "==" => metadata.size == value,
                ">" => metadata.size > value,
                "<" => metadata.size < value,
                ">=" => metadata.size >= value,
                "<=" => metadata.size <= value,
                _ => false,
            }
        } else {
            false
        }
    }

    fn match_created_at(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => metadata.created_at.is_some(),
            _ => false,
        }
    }

    fn match_modified_at(&self, condition: &Condition, _metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => true, // modified_at is always present
            _ => false,
        }
    }

    fn match_capture_date(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => metadata.capture_date.is_some(),
            _ => false,
        }
    }

    fn match_camera_model(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => metadata.camera_model.is_some(),
            "==" => {
                if let (Some(model), Some(value)) = (&metadata.camera_model, condition.value.as_str()) {
                    model == value
                } else {
                    false
                }
            }
            "contains" => {
                if let (Some(model), Some(value)) = (&metadata.camera_model, condition.value.as_str()) {
                    model.contains(value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn match_gps_latitude(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => metadata.gps_latitude.is_some(),
            _ => false,
        }
    }

    fn match_gps_longitude(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "exists" => metadata.gps_longitude.is_some(),
            _ => false,
        }
    }

    fn match_filename(&self, condition: &Condition, metadata: &FileMetadata) -> bool {
        match condition.operator.as_str() {
            "==" => {
                if let Some(value) = condition.value.as_str() {
                    metadata.filename == value
                } else {
                    false
                }
            }
            "contains" => {
                if let Some(value) = condition.value.as_str() {
                    metadata.filename.contains(value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Apply a rule to metadata and expand template variables
    pub fn apply_rule(&self, rule: &Rule, metadata: &FileMetadata) -> Result<String, String> {
        let mut result = rule.destination_pattern.clone();

        // Get the date to use (priority: capture_date > created_at > modified_at)
        let date = metadata.capture_date
            .or(metadata.created_at)
            .unwrap_or(metadata.modified_at);

        // Convert SystemTime to DateTime for formatting
        let datetime = chrono::DateTime::<chrono::Local>::from(date);

        // Replace template variables
        result = result.replace("{year}", &datetime.format("%Y").to_string());
        result = result.replace("{month}", &datetime.format("%m").to_string());
        result = result.replace("{day}", &datetime.format("%d").to_string());
        result = result.replace("{extension}", &metadata.extension);
        result = result.replace("{filename}", &Path::new(&metadata.filename).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&metadata.filename));
        
        // Camera model (replace with "Unknown" if not available)
        let camera = metadata.camera_model.as_ref()
            .map(|c| sanitize_path_component(c))
            .unwrap_or_else(|| "Unknown".to_string());
        result = result.replace("{camera}", &camera);

        // File size in MB
        let size_mb = metadata.size / (1024 * 1024);
        result = result.replace("{size_mb}", &size_mb.to_string());

        Ok(result)
    }
}

/// Sanitize a string to be safe for use as a path component
fn sanitize_path_component(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
