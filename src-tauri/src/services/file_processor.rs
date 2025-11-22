use crate::models::{ProcessResult, OperationType};
use crate::services::{RuleEngine, MetadataExtractor, FileOperations};
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

pub struct FileProcessor {
    rule_engine: RuleEngine,
    metadata_extractor: Box<dyn MetadataExtractor>,
    file_ops: Box<dyn FileOperations>,
    default_destination: Option<String>,
    progress_callback: Option<ProgressCallback>,
    preview_mode: bool,
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
            default_destination: None,
            progress_callback: None,
            preview_mode: false,
        }
    }

    /// Set the default destination for files that don't match any rule
    pub fn set_default_destination(&mut self, destination: String) {
        self.default_destination = Some(destination);
    }

    /// Set a progress callback to receive updates during processing
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    /// Enable or disable preview mode
    /// In preview mode, no actual file operations are performed
    pub fn set_preview_mode(&mut self, enabled: bool) {
        self.preview_mode = enabled;
    }

    /// Get the current preview mode status
    pub fn is_preview_mode(&self) -> bool {
        self.preview_mode
    }

    /// Process multiple files or directories
    /// Recursively processes directories and returns results for all files
    pub fn process_files(&mut self, paths: Vec<String>) -> Vec<ProcessResult> {
        info!("Starting to process {} paths", paths.len());
        
        // Collect all files (expanding directories recursively)
        let all_files = self.collect_files(paths);
        let total_files = all_files.len();
        
        info!("Found {} files to process", total_files);
        
        // Process each file
        let mut results = Vec::new();
        for (index, file_path) in all_files.iter().enumerate() {
            // Call progress callback if set
            if let Some(ref callback) = self.progress_callback {
                callback(index + 1, total_files, file_path);
            }
            
            let result = self.process_file(file_path);
            
            // Log the result
            if result.success {
                info!(
                    "Successfully processed: {} -> {}",
                    result.source_path,
                    result.destination_path.as_ref().unwrap_or(&"(no destination)".to_string())
                );
            } else {
                warn!(
                    "Failed to process {}: {}",
                    result.source_path,
                    result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
                );
            }
            
            results.push(result);
        }
        
        // Log summary
        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.len() - success_count;
        info!(
            "Processing complete: {} succeeded, {} failed",
            success_count, failure_count
        );
        
        results
    }

    /// Process a single file
    pub fn process_file(&mut self, file_path: &str) -> ProcessResult {
        let source_path = file_path.to_string();
        
        // Extract metadata
        let metadata = match self.metadata_extractor.extract(file_path) {
            Ok(meta) => meta,
            Err(e) => {
                error!("Failed to extract metadata from {}: {}", file_path, e);
                return ProcessResult {
                    source_path,
                    destination_path: None,
                    success: false,
                    error_message: Some(format!("Metadata extraction failed: {}", e)),
                    matched_rule: None,
                };
            }
        };
        
        // Find matching rule
        let matched_rule = self.rule_engine.find_matching_rule(&metadata);
        
        // Determine destination path
        let (destination_pattern, operation, rule_name) = match matched_rule {
            Some(rule) => {
                (rule.destination_pattern.clone(), rule.operation.clone(), Some(rule.name.clone()))
            }
            None => {
                // No rule matched - use default destination if set
                match &self.default_destination {
                    Some(default) => {
                        (default.clone(), OperationType::Move, None)
                    }
                    None => {
                        // No default destination - skip this file
                        info!("No rule matched for {} and no default destination set, skipping", file_path);
                        return ProcessResult {
                            source_path,
                            destination_path: None,
                            success: true,
                            error_message: None,
                            matched_rule: None,
                        };
                    }
                }
            }
        };
        
        // Apply rule to get destination path
        let dest_path = match matched_rule {
            Some(rule) => {
                match self.rule_engine.apply_rule(rule, &metadata) {
                    Ok(path) => path,
                    Err(e) => {
                        error!("Failed to apply rule for {}: {}", file_path, e);
                        return ProcessResult {
                            source_path,
                            destination_path: None,
                            success: false,
                            error_message: Some(format!("Rule application failed: {}", e)),
                            matched_rule: rule_name,
                        };
                    }
                }
            }
            None => destination_pattern,
        };
        
        // Construct full destination path with filename
        let dest_full_path = self.construct_destination_path(&dest_path, &metadata.filename);
        
        // In preview mode, skip actual file operations
        if self.preview_mode {
            info!("Preview mode: {} would be moved/copied to {}", file_path, dest_full_path);
            return ProcessResult {
                source_path,
                destination_path: Some(dest_full_path),
                success: true,
                error_message: None,
                matched_rule: rule_name,
            };
        }
        
        // Perform file operation
        let operation_result = match operation {
            OperationType::Move => {
                info!("Moving {} to {}", file_path, dest_full_path);
                self.file_ops.move_file(file_path, &dest_full_path)
            }
            OperationType::Copy => {
                info!("Copying {} to {}", file_path, dest_full_path);
                self.file_ops.copy_file(file_path, &dest_full_path)
            }
        };
        
        match operation_result {
            Ok(_) => {
                ProcessResult {
                    source_path,
                    destination_path: Some(dest_full_path),
                    success: true,
                    error_message: None,
                    matched_rule: rule_name,
                }
            }
            Err(e) => {
                error!("File operation failed for {}: {}", file_path, e);
                ProcessResult {
                    source_path,
                    destination_path: Some(dest_full_path),
                    success: false,
                    error_message: Some(format!("File operation failed: {}", e)),
                    matched_rule: rule_name,
                }
            }
        }
    }

    /// Collect all files from the given paths, recursively expanding directories
    fn collect_files(&self, paths: Vec<String>) -> Vec<String> {
        let mut all_files = Vec::new();
        
        for path_str in paths {
            let path = Path::new(&path_str);
            
            if path.is_file() {
                all_files.push(path_str);
            } else if path.is_dir() {
                // Recursively collect files from directory
                match self.collect_files_from_directory(path) {
                    Ok(files) => all_files.extend(files),
                    Err(e) => {
                        warn!("Failed to read directory {}: {}", path_str, e);
                    }
                }
            } else {
                warn!("Path is neither file nor directory: {}", path_str);
            }
        }
        
        all_files
    }

    /// Recursively collect all files from a directory
    fn collect_files_from_directory(&self, dir: &Path) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            } else if path.is_dir() {
                // Recursively process subdirectory
                match self.collect_files_from_directory(&path) {
                    Ok(subfiles) => files.extend(subfiles),
                    Err(e) => {
                        warn!("Failed to process subdirectory {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(files)
    }

    /// Construct the full destination path by combining the destination directory with the filename
    fn construct_destination_path(&self, dest_dir: &str, filename: &str) -> String {
        let dest_path = PathBuf::from(dest_dir);
        let full_path = dest_path.join(filename);
        
        full_path.to_string_lossy().to_string()
    }
}
