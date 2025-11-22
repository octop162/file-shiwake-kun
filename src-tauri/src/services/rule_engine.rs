use crate::models::{Rule, FileMetadata};

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    pub fn find_matching_rule(&self, _metadata: &FileMetadata) -> Option<&Rule> {
        // TODO: Implement in later tasks
        None
    }

    pub fn apply_rule(&self, _rule: &Rule, _metadata: &FileMetadata) -> Result<String, String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }
}
