use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub mod_time: SystemTime,
}

pub trait FileOperations: Send + Sync {
    fn move_file(&self, source: &str, dest: &str) -> Result<(), String>;
    fn copy_file(&self, source: &str, dest: &str) -> Result<(), String>;
    fn exists(&self, path: &str) -> bool;
    fn create_dir(&self, path: &str) -> Result<(), String>;
    fn get_file_info(&self, path: &str) -> Result<FileInfo, String>;
}

pub struct DefaultFileOperations;

impl FileOperations for DefaultFileOperations {
    fn move_file(&self, _source: &str, _dest: &str) -> Result<(), String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }

    fn copy_file(&self, _source: &str, _dest: &str) -> Result<(), String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }

    fn exists(&self, _path: &str) -> bool {
        // TODO: Implement in later tasks
        false
    }

    fn create_dir(&self, _path: &str) -> Result<(), String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }

    fn get_file_info(&self, _path: &str) -> Result<FileInfo, String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }
}
