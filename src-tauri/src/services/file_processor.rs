use crate::models::{ProcessResult, OperationType, ConflictResolution, FileInfo};
use crate::services::{RuleEngine, MetadataExtractor, FileOperations};
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

/// Callback type for conflict resolution
/// Parameters: source_file_info, dest_file_info
/// Returns: ConflictResolution decision
pub type ConflictCallback = Box<dyn Fn(&FileInfo, &FileInfo) -> ConflictResolution + Send>;

pub struct FileProcessor {
    rule_engine: RuleEngine,
    metadata_extractor: Box<dyn MetadataExtractor>,
    file_ops: Box<dyn FileOperations>,
    default_destination: Option<String>,
    progress_callback: Option<ProgressCallback>,
    conflict_callback: Option<ConflictCallback>,
    preview_mode: bool,
    conflict_policy: Option<ConflictResolution>,
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
            conflict_callback: None,
            preview_mode: false,
            conflict_policy: None,
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

    /// Set a conflict callback to handle file conflicts
    pub fn set_conflict_callback(&mut self, callback: ConflictCallback) {
        self.conflict_callback = Some(callback);
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

    /// Set a persistent conflict policy (for "apply to all" functionality)
    pub fn set_conflict_policy(&mut self, policy: Option<ConflictResolution>) {
        self.conflict_policy = policy;
    }

    /// Get the current conflict policy
    pub fn get_conflict_policy(&self) -> Option<ConflictResolution> {
        self.conflict_policy
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
        
        // Check for conflicts and resolve if necessary
        let final_dest_path = match self.handle_conflict(file_path, &dest_full_path) {
            Ok(path) => path,
            Err(e) => {
                // Conflict resolution failed or user chose to skip
                return ProcessResult {
                    source_path,
                    destination_path: Some(dest_full_path),
                    success: false,
                    error_message: Some(e),
                    matched_rule: rule_name,
                };
            }
        };
        
        // Perform file operation
        let operation_result = match operation {
            OperationType::Move => {
                info!("Moving {} to {}", file_path, final_dest_path);
                self.file_ops.move_file(file_path, &final_dest_path)
            }
            OperationType::Copy => {
                info!("Copying {} to {}", file_path, final_dest_path);
                self.file_ops.copy_file(file_path, &final_dest_path)
            }
        };
        
        match operation_result {
            Ok(_) => {
                ProcessResult {
                    source_path,
                    destination_path: Some(final_dest_path),
                    success: true,
                    error_message: None,
                    matched_rule: rule_name,
                }
            }
            Err(e) => {
                error!("File operation failed for {}: {}", file_path, e);
                ProcessResult {
                    source_path,
                    destination_path: Some(final_dest_path),
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

    /// Handle file conflicts by checking if destination exists and resolving accordingly
    /// Returns the final destination path to use, or an error if the operation should be skipped
    fn handle_conflict(&mut self, source_path: &str, dest_path: &str) -> Result<String, String> {
        // Check if destination file already exists
        if !self.file_ops.exists(dest_path) {
            // No conflict, proceed with original destination
            return Ok(dest_path.to_string());
        }

        info!("Conflict detected: destination file already exists at {}", dest_path);

        // Get file information for both files
        let source_info = self.file_ops.get_file_info(source_path)
            .map_err(|e| format!("Failed to get source file info: {}", e))?;
        
        let dest_info = self.file_ops.get_file_info(dest_path)
            .map_err(|e| format!("Failed to get destination file info: {}", e))?;

        // Determine conflict resolution
        let resolution = self.resolve_conflict(&source_info, &dest_info)?;

        // Apply the resolution
        match resolution {
            ConflictResolution::Overwrite | ConflictResolution::OverwriteAll => {
                // Update policy if "All" variant
                if resolution == ConflictResolution::OverwriteAll {
                    self.conflict_policy = Some(ConflictResolution::OverwriteAll);
                }
                
                info!("Overwriting existing file at {}", dest_path);
                // Return the original path - the file operation will handle the overwrite
                Ok(dest_path.to_string())
            }
            ConflictResolution::Skip | ConflictResolution::SkipAll => {
                // Update policy if "All" variant
                if resolution == ConflictResolution::SkipAll {
                    self.conflict_policy = Some(ConflictResolution::SkipAll);
                }
                
                info!("Skipping file due to conflict at {}", dest_path);
                Err("File skipped due to conflict".to_string())
            }
            ConflictResolution::Rename | ConflictResolution::RenameAll => {
                // Update policy if "All" variant
                if resolution == ConflictResolution::RenameAll {
                    self.conflict_policy = Some(ConflictResolution::RenameAll);
                }
                
                // Generate a new unique filename
                let renamed_path = self.generate_unique_filename(dest_path);
                info!("Renaming to avoid conflict: {}", renamed_path);
                Ok(renamed_path)
            }
        }
    }

    /// Resolve a conflict by checking policy or calling the conflict callback
    fn resolve_conflict(&self, source_info: &FileInfo, dest_info: &FileInfo) -> Result<ConflictResolution, String> {
        // Check if we have a persistent policy set (from "apply to all")
        if let Some(policy) = self.conflict_policy {
            return Ok(policy);
        }

        // Call the conflict callback if available
        if let Some(ref callback) = self.conflict_callback {
            let resolution = callback(source_info, dest_info);
            return Ok(resolution);
        }

        // No callback set - default to skip
        warn!("No conflict resolution callback set, defaulting to Skip");
        Ok(ConflictResolution::Skip)
    }

    /// Generate a unique filename by appending a number to the base name
    fn generate_unique_filename(&self, original_path: &str) -> String {
        let path = Path::new(original_path);
        let parent = path.parent();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str());

        let mut counter = 1;
        loop {
            let new_name = if let Some(ext) = extension {
                format!("{}_{}.{}", stem, counter, ext)
            } else {
                format!("{}_{}", stem, counter)
            };

            let new_path = if let Some(p) = parent {
                p.join(&new_name)
            } else {
                PathBuf::from(&new_name)
            };

            let new_path_str = new_path.to_string_lossy().to_string();
            
            if !self.file_ops.exists(&new_path_str) {
                return new_path_str;
            }

            counter += 1;
            
            // Safety check to prevent infinite loop
            if counter > 10000 {
                warn!("Failed to generate unique filename after 10000 attempts");
                return format!("{}_{}", original_path, counter);
            }
        }
    }
}
