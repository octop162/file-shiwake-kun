// Data models module
pub mod rule;
pub mod config;
pub mod metadata;
pub mod process_result;
pub mod file_info;
pub mod conflict_resolution;

#[cfg(test)]
mod tests;

pub use rule::{Rule, Condition, OperationType};
pub use config::Config;
pub use metadata::FileMetadata;
pub use process_result::ProcessResult;
pub use file_info::FileInfo;
pub use conflict_resolution::ConflictResolution;
