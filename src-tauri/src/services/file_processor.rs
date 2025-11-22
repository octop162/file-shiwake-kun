use crate::models::ProcessResult;
use crate::services::{RuleEngine, MetadataExtractor, FileOperations};

pub struct FileProcessor {
    rule_engine: RuleEngine,
    metadata_extractor: Box<dyn MetadataExtractor>,
    file_ops: Box<dyn FileOperations>,
}

impl FileProcessor {
    pub fn new(
        rule_engine: RuleEngine,
        metadata_extractor: Box<dyn MetadataExtractor>,
        file_ops: Box<dyn FileOperations>,
    ) -> Self {
        Self {
            rule_engine,
            metadata_extractor,
            file_ops,
        }
    }

    pub fn process_files(&mut self, _files: Vec<String>) -> Vec<ProcessResult> {
        // TODO: Implement in later tasks
        vec![]
    }

    pub fn process_file(&mut self, _file: &str) -> ProcessResult {
        // TODO: Implement in later tasks
        ProcessResult {
            source_path: String::new(),
            destination_path: None,
            success: false,
            error_message: Some("Not implemented yet".to_string()),
            matched_rule: None,
        }
    }
}
