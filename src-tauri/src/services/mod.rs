// Services module
pub mod metadata_extractor;
pub mod file_operations;
pub mod rule_engine;
pub mod file_processor;
pub mod config_manager;

#[cfg(test)]
mod config_manager_test;

pub use metadata_extractor::{MetadataExtractor, DefaultMetadataExtractor};
pub use file_operations::{FileOperations, DefaultFileOperations, FileInfo};
pub use rule_engine::RuleEngine;
pub use file_processor::FileProcessor;
pub use config_manager::ConfigManager;
