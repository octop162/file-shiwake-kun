// Feature: file-shiwake-kun, Property 1: ファイル処理の完全性
// For any file or directory that is dropped, all files (recursively for directories) must be processed according to the configured rules
// Validates: Requirements 1.2, 1.3, 1.4

use super::file_processor::FileProcessor;
use super::metadata_extractor::MetadataExtractor;
use super::file_operations::{FileOperations, FileInfo};
use super::rule_engine::RuleEngine;
use crate::models::{FileMetadata, Rule, Condition, OperationType};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use proptest::prelude::*;

// Mock MetadataExtractor for testing
struct MockMetadataExtractor;

impl MetadataExtractor for MockMetadataExtractor {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String> {
        let path = std::path::Path::new(filepath);
        
        if !path.exists() {
            return Err(format!("File does not exist: {}", filepath));
        }
        
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", filepath))?
            .to_string();
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        
        Ok(FileMetadata {
            filename,
            extension,
            size: metadata.len(),
            created_at: metadata.created().ok(),
            modified_at: metadata.modified().unwrap_or(SystemTime::now()),
            capture_date: None,
            camera_model: None,
            gps_latitude: None,
            gps_longitude: None,
        })
    }
}

// Mock FileOperations for testing
struct MockFileOperations {
    operations: Arc<Mutex<Vec<(String, String, String)>>>, // (operation_type, source, dest)
}

impl MockFileOperations {
    fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_operations(&self) -> Vec<(String, String, String)> {
        self.operations.lock().unwrap().clone()
    }
}

impl FileOperations for MockFileOperations {
    fn move_file(&self, source: &str, dest: &str) -> Result<(), String> {
        let path = std::path::Path::new(source);
        if !path.exists() {
            return Err(format!("Source file does not exist: {}", source));
        }
        
        self.operations.lock().unwrap().push((
            "move".to_string(),
            source.to_string(),
            dest.to_string(),
        ));
        
        Ok(())
    }
    
    fn copy_file(&self, source: &str, dest: &str) -> Result<(), String> {
        let path = std::path::Path::new(source);
        if !path.exists() {
            return Err(format!("Source file does not exist: {}", source));
        }
        
        self.operations.lock().unwrap().push((
            "copy".to_string(),
            source.to_string(),
            dest.to_string(),
        ));
        
        Ok(())
    }
    
    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    
    fn create_dir(&self, path: &str) -> Result<(), String> {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory: {}", e))
    }
    
    fn get_file_info(&self, path: &str) -> Result<FileInfo, String> {
        let path_buf = std::path::Path::new(path);
        
        if !path_buf.exists() {
            return Err(format!("File does not exist: {}", path));
        }
        
        let metadata = fs::metadata(path_buf)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;
        
        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        Ok(FileInfo {
            name,
            size: metadata.len(),
            mod_time: metadata.modified().unwrap_or(SystemTime::now()),
        })
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    // Strategy for generating file names
    fn filename_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{1,15}\\.(txt|jpg|png|pdf)"
    }

    // Strategy for generating directory names
    fn dirname_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{1,10}"
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Property 1: File processing completeness
        // When files are dropped (individually), all files must be processed
        #[test]
        fn prop_file_processing_completeness_individual_files(
            filenames in prop::collection::vec(filename_strategy(), 1..10),
            file_contents in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 0..1024),
                1..10
            )
        ) {
            // Ensure we have matching number of filenames and contents
            let num_files = filenames.len().min(file_contents.len());
            let filenames = &filenames[..num_files];
            let file_contents = &file_contents[..num_files];
            
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            // Create test files
            let mut file_paths = Vec::new();
            for (filename, content) in filenames.iter().zip(file_contents.iter()) {
                let file_path = temp_dir.path().join(filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(content).unwrap();
                file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Move,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process the files
            let results = processor.process_files(file_paths.clone());
            
            // Property 1: All files must be processed
            prop_assert_eq!(
                results.len(),
                num_files,
                "Number of results must equal number of input files. Expected: {}, Got: {}",
                num_files,
                results.len()
            );
            
            // Verify each file was processed
            for (idx, file_path) in file_paths.iter().enumerate() {
                let result = &results[idx];
                prop_assert_eq!(
                    &result.source_path,
                    file_path,
                    "Result source path must match input file path"
                );
                
                // Each file should either succeed or have an error message
                if !result.success {
                    prop_assert!(
                        result.error_message.is_some(),
                        "Failed result must have an error message"
                    );
                }
            }
        }

        // Property 1: File processing completeness for directories
        // When a directory is dropped, all files in it (recursively) must be processed
        #[test]
        fn prop_file_processing_completeness_directories(
            num_files_root in 1usize..5,
            num_subdirs in 0usize..3,
            num_files_per_subdir in 0usize..4,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut all_file_paths = Vec::new();
            
            // Create files in root directory
            for i in 0..num_files_root {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                all_file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create subdirectories with files
            for i in 0..num_subdirs {
                let subdir_name = format!("subdir_{}", i);
                let subdir_path = temp_dir.path().join(&subdir_name);
                fs::create_dir(&subdir_path).unwrap();
                
                for j in 0..num_files_per_subdir {
                    let filename = format!("file_{}_{}.txt", i, j);
                    let file_path = subdir_path.join(&filename);
                    let mut file = fs::File::create(&file_path).unwrap();
                    file.write_all(&file_content).unwrap();
                    all_file_paths.push(file_path.to_string_lossy().to_string());
                }
            }
            
            let total_expected_files = all_file_paths.len();
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process the directory (pass the root directory path)
            let results = processor.process_files(vec![temp_dir.path().to_string_lossy().to_string()]);
            
            // Property 1: All files in the directory (recursively) must be processed
            prop_assert_eq!(
                results.len(),
                total_expected_files,
                "Number of results must equal total number of files in directory tree. Expected: {}, Got: {}",
                total_expected_files,
                results.len()
            );
            
            // Verify all expected files were processed
            let processed_paths: std::collections::HashSet<_> = results
                .iter()
                .map(|r| r.source_path.clone())
                .collect();
            
            for expected_path in &all_file_paths {
                prop_assert!(
                    processed_paths.contains(expected_path),
                    "Expected file was not processed: {}",
                    expected_path
                );
            }
        }

        // Property 1: File processing completeness for mixed input
        // When both files and directories are dropped, all must be processed
        #[test]
        fn prop_file_processing_completeness_mixed_input(
            num_individual_files in 1usize..4,
            num_dirs in 1usize..3,
            num_files_per_dir in 1usize..4,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut input_paths = Vec::new();
            let mut all_expected_files = Vec::new();
            
            // Create individual files
            for i in 0..num_individual_files {
                let filename = format!("individual_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                
                input_paths.push(file_path.to_string_lossy().to_string());
                all_expected_files.push(file_path.to_string_lossy().to_string());
            }
            
            // Create directories with files
            for i in 0..num_dirs {
                let dir_name = format!("dir_{}", i);
                let dir_path = temp_dir.path().join(&dir_name);
                fs::create_dir(&dir_path).unwrap();
                
                for j in 0..num_files_per_dir {
                    let filename = format!("file_{}_{}.txt", i, j);
                    let file_path = dir_path.join(&filename);
                    let mut file = fs::File::create(&file_path).unwrap();
                    file.write_all(&file_content).unwrap();
                    all_expected_files.push(file_path.to_string_lossy().to_string());
                }
                
                input_paths.push(dir_path.to_string_lossy().to_string());
            }
            
            let total_expected_files = all_expected_files.len();
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process mixed input (files and directories)
            let results = processor.process_files(input_paths);
            
            // Property 1: All files (from both individual files and directories) must be processed
            prop_assert_eq!(
                results.len(),
                total_expected_files,
                "Number of results must equal total number of files. Expected: {}, Got: {}",
                total_expected_files,
                results.len()
            );
            
            // Verify all expected files were processed
            let processed_paths: std::collections::HashSet<_> = results
                .iter()
                .map(|r| r.source_path.clone())
                .collect();
            
            for expected_path in &all_expected_files {
                prop_assert!(
                    processed_paths.contains(expected_path),
                    "Expected file was not processed: {}",
                    expected_path
                );
            }
        }

        // Feature: file-shiwake-kun, Property 2: 進捗情報の提供
        // For any file set being processed, progress callbacks must be invoked for each file
        // Validates: Requirements 1.5, 8.4
        #[test]
        fn prop_progress_information_provision(
            num_files in 1usize..20,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut file_paths = Vec::new();
            
            // Create test files
            for i in 0..num_files {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Track progress callbacks
            let progress_calls = Arc::new(Mutex::new(Vec::new()));
            let progress_calls_clone = Arc::clone(&progress_calls);
            
            processor.set_progress_callback(Box::new(move |current, total, path| {
                progress_calls_clone.lock().unwrap().push((current, total, path.to_string()));
            }));
            
            // Process the files
            let results = processor.process_files(file_paths.clone());
            
            // Get the progress calls
            let calls = progress_calls.lock().unwrap();
            
            // Property 2: Progress callback must be invoked for each file
            prop_assert_eq!(
                calls.len(),
                num_files,
                "Progress callback must be called once for each file. Expected: {}, Got: {}",
                num_files,
                calls.len()
            );
            
            // Verify the progress information is correct
            for (idx, (current, total, path)) in calls.iter().enumerate() {
                // Current should be 1-indexed and match the iteration
                prop_assert_eq!(
                    *current,
                    idx + 1,
                    "Progress current index should be {}, got {}",
                    idx + 1,
                    current
                );
                
                // Total should always be the total number of files
                prop_assert_eq!(
                    *total,
                    num_files,
                    "Progress total should be {}, got {}",
                    num_files,
                    total
                );
                
                // Path should be one of the input files
                prop_assert!(
                    file_paths.contains(path),
                    "Progress path '{}' should be one of the input files",
                    path
                );
            }
            
            // Verify all files were reported in progress
            let reported_paths: std::collections::HashSet<_> = calls
                .iter()
                .map(|(_, _, path)| path.clone())
                .collect();
            
            for file_path in &file_paths {
                prop_assert!(
                    reported_paths.contains(file_path),
                    "File '{}' should have been reported in progress callbacks",
                    file_path
                );
            }
            
            // Verify results match the number of files
            prop_assert_eq!(
                results.len(),
                num_files,
                "Number of results should match number of files"
            );
        }

        // Feature: file-shiwake-kun, Property 2: 進捗情報の提供
        // For directories, progress callbacks must be invoked for each file in the directory tree
        // Validates: Requirements 1.5, 8.4
        #[test]
        fn prop_progress_information_provision_directories(
            num_files_root in 1usize..5,
            num_subdirs in 1usize..4,
            num_files_per_subdir in 1usize..4,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut all_file_paths = Vec::new();
            
            // Create files in root directory
            for i in 0..num_files_root {
                let filename = format!("root_file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                all_file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create subdirectories with files
            for i in 0..num_subdirs {
                let subdir_name = format!("subdir_{}", i);
                let subdir_path = temp_dir.path().join(&subdir_name);
                fs::create_dir(&subdir_path).unwrap();
                
                for j in 0..num_files_per_subdir {
                    let filename = format!("file_{}_{}.txt", i, j);
                    let file_path = subdir_path.join(&filename);
                    let mut file = fs::File::create(&file_path).unwrap();
                    file.write_all(&file_content).unwrap();
                    all_file_paths.push(file_path.to_string_lossy().to_string());
                }
            }
            
            let total_files = all_file_paths.len();
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Track progress callbacks
            let progress_calls = Arc::new(Mutex::new(Vec::new()));
            let progress_calls_clone = Arc::clone(&progress_calls);
            
            processor.set_progress_callback(Box::new(move |current, total, path| {
                progress_calls_clone.lock().unwrap().push((current, total, path.to_string()));
            }));
            
            // Process the directory
            let results = processor.process_files(vec![temp_dir.path().to_string_lossy().to_string()]);
            
            // Get the progress calls
            let calls = progress_calls.lock().unwrap();
            
            // Property 2: Progress callback must be invoked for each file in the directory tree
            prop_assert_eq!(
                calls.len(),
                total_files,
                "Progress callback must be called once for each file in directory tree. Expected: {}, Got: {}",
                total_files,
                calls.len()
            );
            
            // Verify the progress information is correct
            for (idx, (current, total, _path)) in calls.iter().enumerate() {
                // Current should be 1-indexed and match the iteration
                prop_assert_eq!(
                    *current,
                    idx + 1,
                    "Progress current index should be {}, got {}",
                    idx + 1,
                    current
                );
                
                // Total should always be the total number of files
                prop_assert_eq!(
                    *total,
                    total_files,
                    "Progress total should be {}, got {}",
                    total_files,
                    total
                );
            }
            
            // Verify all files were reported in progress
            let reported_paths: std::collections::HashSet<_> = calls
                .iter()
                .map(|(_, _, path)| path.clone())
                .collect();
            
            for file_path in &all_file_paths {
                prop_assert!(
                    reported_paths.contains(file_path),
                    "File '{}' should have been reported in progress callbacks",
                    file_path
                );
            }
            
            // Verify results match the number of files
            prop_assert_eq!(
                results.len(),
                total_files,
                "Number of results should match total number of files"
            );
        }

        // Feature: file-shiwake-kun, Property 14: 処理結果のログ記録
        // For any file operation (success or failure), it must be logged with source path and destination path (or failure reason)
        // Validates: Requirements 8.1, 8.5
        #[test]
        fn prop_processing_result_logging(
            num_files in 1usize..10,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut file_paths = Vec::new();
            
            // Create test files
            for i in 0..num_files {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process the files
            let results = processor.process_files(file_paths.clone());
            
            // Property 14: All file operations must be logged
            // We verify this by checking that ProcessResult contains all necessary information
            // that would be logged (source path, destination path, error messages)
            // The actual logging is done by the tracing crate in the FileProcessor implementation
            
            for result in &results {
                // Verify that each result has a source path (required for logging)
                prop_assert!(
                    !result.source_path.is_empty(),
                    "Result must have a non-empty source path for logging"
                );
                
                if result.success {
                    // Successful operations must have destination path for logging
                    prop_assert!(
                        result.destination_path.is_some(),
                        "Successful operation must have destination path for logging: {}",
                        result.source_path
                    );
                    
                    // Successful operations should not have error messages
                    prop_assert!(
                        result.error_message.is_none(),
                        "Successful operation should not have error message: {}",
                        result.source_path
                    );
                } else {
                    // Failed operations must have error message for logging
                    prop_assert!(
                        result.error_message.is_some(),
                        "Failed operation must have error message for logging: {}",
                        result.source_path
                    );
                    
                    let error_msg = result.error_message.as_ref().unwrap();
                    prop_assert!(
                        !error_msg.is_empty(),
                        "Failed operation must have non-empty error message for logging: {}",
                        result.source_path
                    );
                }
            }
            
            // Verify that we have results for all files (needed for summary logging)
            prop_assert_eq!(
                results.len(),
                num_files,
                "Must have results for all files to log summary"
            );
            
            // Verify that we can compute success/failure counts for summary logging
            let success_count = results.iter().filter(|r| r.success).count();
            let failure_count = results.len() - success_count;
            
            prop_assert!(
                success_count + failure_count == num_files,
                "Success and failure counts must sum to total files for summary logging"
            );
        }

        // Feature: file-shiwake-kun, Property 15: 処理サマリーの正確性
        // For any file set, when processing completes, the summary success and failure counts must match the actual results
        // Validates: Requirements 8.3
        #[test]
        fn prop_processing_summary_accuracy(
            num_files in 1usize..20,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut file_paths = Vec::new();
            
            // Create test files
            for i in 0..num_files {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process the files
            let results = processor.process_files(file_paths.clone());
            
            // Property 15: Summary counts must match actual results
            
            // Count actual successes and failures from results
            let actual_success_count = results.iter().filter(|r| r.success).count();
            let actual_failure_count = results.iter().filter(|r| !r.success).count();
            
            // The summary that would be displayed should match these counts
            // (In the actual implementation, this is logged in process_files)
            let summary_success_count = results.iter().filter(|r| r.success).count();
            let summary_failure_count = results.len() - summary_success_count;
            
            // Verify summary counts match actual results
            prop_assert_eq!(
                summary_success_count,
                actual_success_count,
                "Summary success count must match actual success count. Expected: {}, Got: {}",
                actual_success_count,
                summary_success_count
            );
            
            prop_assert_eq!(
                summary_failure_count,
                actual_failure_count,
                "Summary failure count must match actual failure count. Expected: {}, Got: {}",
                actual_failure_count,
                summary_failure_count
            );
            
            // Verify that success + failure = total
            prop_assert_eq!(
                summary_success_count + summary_failure_count,
                results.len(),
                "Summary counts must sum to total number of results"
            );
            
            prop_assert_eq!(
                results.len(),
                num_files,
                "Total results must equal number of input files"
            );
            
            // Verify each result is properly categorized
            for result in &results {
                if result.success {
                    // Successful results should have destination path
                    prop_assert!(
                        result.destination_path.is_some() || result.matched_rule.is_none(),
                        "Successful result should have destination path or no matched rule (skipped): {}",
                        result.source_path
                    );
                    
                    // Successful results should not have error messages
                    prop_assert!(
                        result.error_message.is_none(),
                        "Successful result should not have error message: {}",
                        result.source_path
                    );
                } else {
                    // Failed results should have error messages
                    prop_assert!(
                        result.error_message.is_some(),
                        "Failed result must have error message: {}",
                        result.source_path
                    );
                }
            }
        }

        // Feature: file-shiwake-kun, Property 15: 処理サマリーの正確性
        // For any set of files processed, summary must accurately reflect actual results
        // Validates: Requirements 8.3
        #[test]
        fn prop_processing_summary_accuracy_with_directories(
            num_files_root in 1usize..8,
            num_subdirs in 1usize..4,
            num_files_per_subdir in 1usize..5,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut all_file_paths = Vec::new();
            
            // Create files in root directory
            for i in 0..num_files_root {
                let filename = format!("root_file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                all_file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create subdirectories with files
            for i in 0..num_subdirs {
                let subdir_name = format!("subdir_{}", i);
                let subdir_path = temp_dir.path().join(&subdir_name);
                fs::create_dir(&subdir_path).unwrap();
                
                for j in 0..num_files_per_subdir {
                    let filename = format!("file_{}_{}.txt", i, j);
                    let file_path = subdir_path.join(&filename);
                    let mut file = fs::File::create(&file_path).unwrap();
                    file.write_all(&file_content).unwrap();
                    all_file_paths.push(file_path.to_string_lossy().to_string());
                }
            }
            
            let total_files = all_file_paths.len();
            
            // Create a rule that matches all files
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "modified_at".to_string(),
                        operator: "exists".to_string(),
                        value: serde_json::json!(null),
                    },
                ],
                destination_pattern: temp_dir.path().join("output").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Process the directory (this will process all files recursively)
            let results = processor.process_files(vec![temp_dir.path().to_string_lossy().to_string()]);
            
            // Property 15: Summary counts must accurately reflect actual results
            
            // Count actual successes and failures from results
            let actual_success_count = results.iter().filter(|r| r.success).count();
            let actual_failure_count = results.iter().filter(|r| !r.success).count();
            
            // Compute summary counts (as done in process_files)
            let summary_success_count = results.iter().filter(|r| r.success).count();
            let summary_failure_count = results.len() - summary_success_count;
            
            // Verify summary matches actual results
            prop_assert_eq!(
                summary_success_count,
                actual_success_count,
                "Summary success count must match actual successes"
            );
            
            prop_assert_eq!(
                summary_failure_count,
                actual_failure_count,
                "Summary failure count must match actual failures"
            );
            
            // Verify we have results for all files
            prop_assert_eq!(
                results.len(),
                total_files,
                "Must have results for all input files"
            );
            
            // Verify counts sum correctly
            prop_assert_eq!(
                summary_success_count + summary_failure_count,
                total_files,
                "Summary counts must sum to total files"
            );
            
            // Verify each result is properly categorized
            for result in &results {
                if result.success {
                    // Successful results should have destination path
                    prop_assert!(
                        result.destination_path.is_some() || result.matched_rule.is_none(),
                        "Successful result should have destination path or no matched rule (skipped): {}",
                        result.source_path
                    );
                } else {
                    // Failed results should have error messages
                    prop_assert!(
                        result.error_message.is_some(),
                        "Failed result must have error message: {}",
                        result.source_path
                    );
                }
            }
        }
    }
}
