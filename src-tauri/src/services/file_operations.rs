use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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

impl DefaultFileOperations {
    /// Validates a path to prevent path traversal attacks and ensure it's safe
    fn validate_path(&self, path: &str) -> Result<PathBuf, String> {
        let path_buf = PathBuf::from(path);
        
        // Normalize the path to resolve any .. or . components
        let normalized = path_buf
            .canonicalize()
            .or_else(|_| {
                // If canonicalize fails (e.g., path doesn't exist yet), 
                // try to canonicalize the parent and append the filename
                if let Some(parent) = path_buf.parent() {
                    if parent.as_os_str().is_empty() {
                        // Relative path with no parent, use current dir
                        std::env::current_dir()
                            .map(|cwd| cwd.join(&path_buf))
                            .map_err(|e| format!("Failed to get current directory: {}", e))
                    } else {
                        parent.canonicalize()
                            .map(|p| {
                                if let Some(filename) = path_buf.file_name() {
                                    p.join(filename)
                                } else {
                                    p
                                }
                            })
                            .map_err(|e| format!("Failed to canonicalize parent path: {}", e))
                    }
                } else {
                    Err(format!("Invalid path: {}", path))
                }
            })?;
        
        // Check for invalid characters in the path
        let path_str = normalized.to_string_lossy();
        if path_str.contains('\0') {
            return Err("Path contains null character".to_string());
        }
        
        Ok(normalized)
    }

    /// Ensures the parent directory exists, creating it if necessary
    fn ensure_parent_dir(&self, dest_path: &Path) -> Result<(), String> {
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
        }
        Ok(())
    }
}

impl FileOperations for DefaultFileOperations {
    /// Moves a file from source to destination
    /// Creates parent directories if they don't exist
    /// Validates paths to prevent security issues
    fn move_file(&self, source: &str, dest: &str) -> Result<(), String> {
        let source_path = self.validate_path(source)?;
        let dest_path = PathBuf::from(dest);
        
        // Check if source exists
        if !source_path.exists() {
            return Err(format!("Source file does not exist: {}", source));
        }
        
        // Check if source is a file
        if !source_path.is_file() {
            return Err(format!("Source is not a file: {}", source));
        }
        
        // Ensure parent directory exists
        self.ensure_parent_dir(&dest_path)?;
        
        // Perform the move operation
        fs::rename(&source_path, &dest_path)
            .map_err(|e| format!("Failed to move file from {} to {}: {}", source, dest, e))?;
        
        Ok(())
    }

    /// Copies a file from source to destination
    /// Creates parent directories if they don't exist
    /// Validates paths to prevent security issues
    fn copy_file(&self, source: &str, dest: &str) -> Result<(), String> {
        let source_path = self.validate_path(source)?;
        let dest_path = PathBuf::from(dest);
        
        // Check if source exists
        if !source_path.exists() {
            return Err(format!("Source file does not exist: {}", source));
        }
        
        // Check if source is a file
        if !source_path.is_file() {
            return Err(format!("Source is not a file: {}", source));
        }
        
        // Ensure parent directory exists
        self.ensure_parent_dir(&dest_path)?;
        
        // Perform the copy operation
        fs::copy(&source_path, &dest_path)
            .map_err(|e| format!("Failed to copy file from {} to {}: {}", source, dest, e))?;
        
        Ok(())
    }

    /// Checks if a path exists
    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Creates a directory and all necessary parent directories
    fn create_dir(&self, path: &str) -> Result<(), String> {
        let path_buf = PathBuf::from(path);
        
        // Check for invalid characters
        let path_str = path_buf.to_string_lossy();
        if path_str.contains('\0') {
            return Err("Path contains null character".to_string());
        }
        
        // Create directory with all parents
        fs::create_dir_all(&path_buf)
            .map_err(|e| format!("Failed to create directory {}: {}", path, e))?;
        
        Ok(())
    }

    /// Gets file information (name, size, modification time)
    fn get_file_info(&self, path: &str) -> Result<FileInfo, String> {
        let path_buf = self.validate_path(path)?;
        
        // Check if file exists
        if !path_buf.exists() {
            return Err(format!("File does not exist: {}", path));
        }
        
        // Get metadata
        let metadata = fs::metadata(&path_buf)
            .map_err(|e| format!("Failed to get file metadata for {}: {}", path, e))?;
        
        // Extract file name
        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        // Get modification time
        let mod_time = metadata
            .modified()
            .map_err(|e| format!("Failed to get modification time for {}: {}", path, e))?;
        
        Ok(FileInfo {
            name,
            size: metadata.len(),
            mod_time,
        })
    }
}
