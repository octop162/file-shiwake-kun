// Unit tests for preview mode functionality
// These tests verify that preview mode works correctly without performing actual file operations

use super::file_processor::FileProcessor;
use super::metadata_extractor::MetadataExtractor;
use super::file_operations::FileOperations;
use super::rule_engine::RuleEngine;
use crate::models::{FileMetadata, Rule, Condition, OperationType, FileInfo};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

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

// Mock FileOperations that tracks operations
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
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_preview_mode_no_file_operations() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let file1_path = temp_dir.path().join("test1.txt");
        let file2_path = temp_dir.path().join("test2.txt");
        
        fs::File::create(&file1_path).unwrap().write_all(b"test content 1").unwrap();
        fs::File::create(&file2_path).unwrap().write_all(b"test content 2").unwrap();
        
        let file_paths = vec![
            file1_path.to_string_lossy().to_string(),
            file2_path.to_string_lossy().to_string(),
        ];
        
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
        let file_ops_ref = file_ops.get_operations();
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            file_ops,
        );
        
        // Enable preview mode
        processor.set_preview_mode(true);
        
        // Verify preview mode is enabled
        assert!(processor.is_preview_mode());
        
        // Process the files in preview mode
        let results = processor.process_files(file_paths.clone());
        
        // Verify results were generated
        assert_eq!(results.len(), 2);
        
        // Verify all results are successful
        for result in &results {
            assert!(result.success, "Preview should succeed for: {}", result.source_path);
            assert!(result.destination_path.is_some(), "Preview should show destination");
            assert!(result.error_message.is_none(), "Preview should not have errors");
        }
        
        // Verify NO actual file operations were performed
        assert_eq!(
            file_ops_ref.len(),
            0,
            "Preview mode should not perform any file operations"
        );
        
        // Verify original files still exist
        assert!(file1_path.exists(), "Original file should still exist in preview mode");
        assert!(file2_path.exists(), "Original file should still exist in preview mode");
    }

    #[test]
    fn test_preview_mode_shows_matched_rule() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        
        // Create test file
        let file_path = temp_dir.path().join("test.jpg");
        fs::File::create(&file_path).unwrap().write_all(b"fake image").unwrap();
        
        // Create a rule with a specific name
        let rule = Rule {
            id: "photo-rule".to_string(),
            name: "Photo Organization Rule".to_string(),
            priority: 1,
            conditions: vec![
                Condition {
                    field: "extension".to_string(),
                    operator: "==".to_string(),
                    value: serde_json::json!("jpg"),
                },
            ],
            destination_pattern: temp_dir.path().join("photos").to_string_lossy().to_string(),
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
        
        // Enable preview mode
        processor.set_preview_mode(true);
        
        // Process the file
        let results = processor.process_files(vec![file_path.to_string_lossy().to_string()]);
        
        // Verify result shows matched rule
        assert_eq!(results.len(), 1);
        let result = &results[0];
        
        assert!(result.success);
        assert_eq!(result.matched_rule, Some("Photo Organization Rule".to_string()));
        assert!(result.destination_path.is_some());
    }

    #[test]
    fn test_preview_mode_toggle() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        
        // Create test file
        let file_path = temp_dir.path().join("test.txt");
        fs::File::create(&file_path).unwrap().write_all(b"test").unwrap();
        
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
        let file_ops_clone = Arc::new(Mutex::new(file_ops.get_operations()));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            file_ops,
        );
        
        // Initially preview mode should be disabled
        assert!(!processor.is_preview_mode());
        
        // Enable preview mode
        processor.set_preview_mode(true);
        assert!(processor.is_preview_mode());
        
        // Process in preview mode
        let results = processor.process_files(vec![file_path.to_string_lossy().to_string()]);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        
        // Disable preview mode
        processor.set_preview_mode(false);
        assert!(!processor.is_preview_mode());
    }

    // Feature: file-shiwake-kun, Property 16: プレビューモードの非破壊性
    // For any file, when processing is executed in preview mode, no actual file operations should be performed
    // and only the intended destination should be displayed
    // Validates: Requirements 9.1
    
    // Strategy for generating file names
    fn filename_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{1,15}\\.(txt|jpg|png|pdf)"
    }
    
    // Strategy for generating rule names
    fn rule_name_strategy() -> impl Strategy<Value = String> {
        "[A-Z][a-zA-Z0-9 ]{5,30}"
    }
    
    // Strategy for generating file extensions
    fn extension_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(vec!["txt", "jpg", "png", "pdf", "doc", "mp4"])
            .prop_map(|s| s.to_string())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_preview_mode_non_destructiveness(
            num_files in 1usize..10,
            file_contents in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 0..1024),
                1..10
            )
        ) {
            // Ensure we have enough content for the files
            let num_files = num_files.min(file_contents.len());
            let file_contents = &file_contents[..num_files];
            
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            // Create test files with unique names and record their initial state
            let mut file_paths = Vec::new();
            let mut initial_file_states = Vec::new();
            
            for (i, content) in file_contents.iter().enumerate() {
                // Generate unique filename for each file
                let filename = format!("preview_test_file_{}.dat", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(content).unwrap();
                drop(file); // Ensure file is closed
                
                let path_str = file_path.to_string_lossy().to_string();
                file_paths.push(path_str.clone());
                
                // Record initial state
                let metadata = fs::metadata(&file_path).unwrap();
                initial_file_states.push((
                    path_str,
                    metadata.len(),
                    metadata.modified().unwrap(),
                    content.clone(),
                ));
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
            let file_ops_ref = file_ops.operations.clone();
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Enable preview mode
            processor.set_preview_mode(true);
            
            // Process the files in preview mode
            let results = processor.process_files(file_paths.clone());
            
            // Property 16: Preview mode must be non-destructive
            
            // 1. Verify NO actual file operations were performed
            let operations = file_ops_ref.lock().unwrap();
            prop_assert_eq!(
                operations.len(),
                0,
                "Preview mode must not perform any file operations. Found {} operations",
                operations.len()
            );
            drop(operations);
            
            // 2. Verify all original files still exist with unchanged content
            for (original_path, original_size, _original_mod_time, original_content) in &initial_file_states {
                let path = std::path::Path::new(original_path);
                
                prop_assert!(
                    path.exists(),
                    "Original file must still exist in preview mode: {}",
                    original_path
                );
                
                // Verify file size hasn't changed
                let current_metadata = fs::metadata(path).unwrap();
                prop_assert_eq!(
                    current_metadata.len(),
                    *original_size,
                    "File size must not change in preview mode: {}",
                    original_path
                );
                
                // Verify file content hasn't changed
                let current_content = fs::read(path).unwrap();
                prop_assert_eq!(
                    &current_content,
                    original_content,
                    "File content must not change in preview mode: {}",
                    original_path
                );
            }
            
            // 3. Verify results show intended destinations
            prop_assert_eq!(
                results.len(),
                num_files,
                "Preview mode must return results for all files"
            );
            
            for result in &results {
                // Each result should be successful in preview mode
                prop_assert!(
                    result.success,
                    "Preview mode results should be successful: {}",
                    result.source_path
                );
                
                // Each result should show the intended destination
                prop_assert!(
                    result.destination_path.is_some(),
                    "Preview mode must show intended destination: {}",
                    result.source_path
                );
                
                // No error messages in preview mode
                prop_assert!(
                    result.error_message.is_none(),
                    "Preview mode should not have error messages: {}",
                    result.source_path
                );
            }
            
            // 4. Verify destination directory was NOT created
            let dest_dir = temp_dir.path().join("output");
            prop_assert!(
                !dest_dir.exists(),
                "Preview mode must not create destination directories"
            );
        }

        #[test]
        fn prop_preview_mode_non_destructiveness_with_directories(
            num_files_root in 1usize..5,
            num_subdirs in 1usize..3,
            num_files_per_subdir in 1usize..4,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut all_file_paths = Vec::new();
            let mut initial_file_states = Vec::new();
            
            // Create files in root directory
            for i in 0..num_files_root {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                drop(file);
                
                let path_str = file_path.to_string_lossy().to_string();
                all_file_paths.push(path_str.clone());
                
                let metadata = fs::metadata(&file_path).unwrap();
                initial_file_states.push((
                    path_str,
                    metadata.len(),
                    file_content.clone(),
                ));
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
                    drop(file);
                    
                    let path_str = file_path.to_string_lossy().to_string();
                    all_file_paths.push(path_str.clone());
                    
                    let metadata = fs::metadata(&file_path).unwrap();
                    initial_file_states.push((
                        path_str,
                        metadata.len(),
                        file_content.clone(),
                    ));
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
            let file_ops_ref = file_ops.operations.clone();
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Enable preview mode
            processor.set_preview_mode(true);
            
            // Process the directory in preview mode
            let results = processor.process_files(vec![temp_dir.path().to_string_lossy().to_string()]);
            
            // Property 16: Preview mode must be non-destructive for directories
            
            // 1. Verify NO actual file operations were performed
            let operations = file_ops_ref.lock().unwrap();
            prop_assert_eq!(
                operations.len(),
                0,
                "Preview mode must not perform any file operations on directory contents. Found {} operations",
                operations.len()
            );
            drop(operations);
            
            // 2. Verify all original files still exist with unchanged content
            for (original_path, original_size, original_content) in &initial_file_states {
                let path = std::path::Path::new(original_path);
                
                prop_assert!(
                    path.exists(),
                    "Original file must still exist in preview mode: {}",
                    original_path
                );
                
                // Verify file size hasn't changed
                let current_metadata = fs::metadata(path).unwrap();
                prop_assert_eq!(
                    current_metadata.len(),
                    *original_size,
                    "File size must not change in preview mode: {}",
                    original_path
                );
                
                // Verify file content hasn't changed
                let current_content = fs::read(path).unwrap();
                prop_assert_eq!(
                    &current_content,
                    original_content,
                    "File content must not change in preview mode: {}",
                    original_path
                );
            }
            
            // 3. Verify results show intended destinations for all files
            prop_assert_eq!(
                results.len(),
                total_files,
                "Preview mode must return results for all files in directory tree"
            );
            
            for result in &results {
                // Each result should be successful in preview mode
                prop_assert!(
                    result.success,
                    "Preview mode results should be successful: {}",
                    result.source_path
                );
                
                // Each result should show the intended destination
                prop_assert!(
                    result.destination_path.is_some(),
                    "Preview mode must show intended destination: {}",
                    result.source_path
                );
            }
            
            // 4. Verify destination directory was NOT created
            let dest_dir = temp_dir.path().join("output");
            prop_assert!(
                !dest_dir.exists(),
                "Preview mode must not create destination directories"
            );
        }

        // Feature: file-shiwake-kun, Property 17: プレビュー結果のルール表示
        // For any file's preview result, the matched rule (or indication that no rule matched) must be displayed
        // Validates: Requirements 9.2
        #[test]
        fn prop_preview_results_show_matched_rule(
            rule_names in prop::collection::vec(rule_name_strategy(), 1..5),
            num_files in 1usize..10
        ) {
            let num_rules = rule_names.len();
            
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            // Create rules that match UNIQUE extensions (to avoid priority conflicts)
            let mut rules = Vec::new();
            let extensions: Vec<String> = (0..num_rules)
                .map(|i| format!("ext{}", i))
                .collect();
            
            for (i, (rule_name, extension)) in rule_names.iter().zip(extensions.iter()).enumerate() {
                let rule = Rule {
                    id: format!("rule-{}", i),
                    name: rule_name.clone(),
                    priority: i as i32 + 1,
                    conditions: vec![
                        Condition {
                            field: "extension".to_string(),
                            operator: "==".to_string(),
                            value: serde_json::json!(extension),
                        },
                    ],
                    destination_pattern: temp_dir.path()
                        .join(format!("output_{}", extension))
                        .to_string_lossy()
                        .to_string(),
                    operation: OperationType::Move,
                };
                rules.push(rule);
            }
            
            // Create test files with various extensions
            let mut file_paths = Vec::new();
            let mut expected_matches = Vec::new();
            
            for i in 0..num_files {
                // Alternate between extensions that match rules and ones that don't
                let extension = if i % 2 == 0 && !extensions.is_empty() {
                    // Use an extension that matches a rule
                    extensions[i % extensions.len()].clone()
                } else {
                    // Use an extension that doesn't match any rule
                    "xyz".to_string()
                };
                
                let filename = format!("file_{}.{}", i, extension);
                let file_path = temp_dir.path().join(&filename);
                
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(b"test content").unwrap();
                drop(file);
                
                let path_str = file_path.to_string_lossy().to_string();
                file_paths.push(path_str);
                
                // Determine expected match
                if i % 2 == 0 && !extensions.is_empty() {
                    let rule_index = i % extensions.len();
                    expected_matches.push(Some(rule_names[rule_index].clone()));
                } else {
                    expected_matches.push(None);
                }
            }
            
            // Create processor with rules
            let rule_engine = RuleEngine::new(rules);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Enable preview mode
            processor.set_preview_mode(true);
            
            // Process the files
            let results = processor.process_files(file_paths.clone());
            
            // Property 17: Preview results must show matched rule information
            
            prop_assert_eq!(
                results.len(),
                num_files,
                "Must return results for all files"
            );
            
            for (i, result) in results.iter().enumerate() {
                // Verify the result shows the matched rule (or None if no match)
                prop_assert_eq!(
                    &result.matched_rule,
                    &expected_matches[i],
                    "Preview result must show correct matched rule for file {}: expected {:?}, got {:?}",
                    i,
                    &expected_matches[i],
                    &result.matched_rule
                );
                
                // If a rule matched, verify destination is shown
                if expected_matches[i].is_some() {
                    prop_assert!(
                        result.destination_path.is_some(),
                        "Preview result must show destination when rule matches for file {}",
                        i
                    );
                    
                    prop_assert!(
                        result.success,
                        "Preview result must be successful when rule matches for file {}",
                        i
                    );
                }
                
                // Verify no error messages in preview mode
                prop_assert!(
                    result.error_message.is_none(),
                    "Preview result should not have error messages for file {}: {:?}",
                    i,
                    result.error_message
                );
            }
        }

        // Additional test: Verify preview results show matched rule with multiple matching rules
        // This tests that the highest priority rule is shown
        #[test]
        fn prop_preview_results_show_highest_priority_rule(
            rule_names in prop::collection::vec(rule_name_strategy(), 2..5),
            num_files in 1usize..8
        ) {
            let num_rules = rule_names.len();
            
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            // Create multiple rules that all match the same extension
            // The rule engine sorts by priority in DESCENDING order (higher number = higher priority)
            // So the LAST rule (with highest priority number) should be matched
            let mut rules = Vec::new();
            for (i, rule_name) in rule_names.iter().enumerate() {
                let rule = Rule {
                    id: format!("rule-{}", i),
                    name: rule_name.clone(),
                    priority: i as i32 + 1, // Higher number = higher priority in the engine
                    conditions: vec![
                        Condition {
                            field: "extension".to_string(),
                            operator: "==".to_string(),
                            value: serde_json::json!("txt"),
                        },
                    ],
                    destination_pattern: temp_dir.path()
                        .join(format!("output_{}", i))
                        .to_string_lossy()
                        .to_string(),
                    operation: OperationType::Copy,
                };
                rules.push(rule);
            }
            
            // Create test files with .txt extension
            let mut file_paths = Vec::new();
            
            for i in 0..num_files {
                let filename = format!("file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(b"test content").unwrap();
                drop(file);
                
                file_paths.push(file_path.to_string_lossy().to_string());
            }
            
            // Create processor with rules
            let rule_engine = RuleEngine::new(rules);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Enable preview mode
            processor.set_preview_mode(true);
            
            // Process the files
            let results = processor.process_files(file_paths);
            
            // Property 17: Preview results must show the highest priority matched rule
            
            prop_assert_eq!(
                results.len(),
                num_files,
                "Must return results for all files"
            );
            
            // The last rule (highest priority number) should be matched for all files
            let expected_rule_name = &rule_names[num_rules - 1];
            
            for (i, result) in results.iter().enumerate() {
                prop_assert_eq!(
                    result.matched_rule.as_ref(),
                    Some(expected_rule_name),
                    "Preview result must show highest priority rule for file {}: expected {:?}, got {:?}",
                    i,
                    Some(expected_rule_name),
                    result.matched_rule
                );
                
                prop_assert!(
                    result.destination_path.is_some(),
                    "Preview result must show destination for file {}",
                    i
                );
                
                prop_assert!(
                    result.success,
                    "Preview result must be successful for file {}",
                    i
                );
            }
        }

        // Feature: file-shiwake-kun, Property 18: プレビューキャンセルの非破壊性
        // For any file set, when preview is cancelled after preview, all files must remain unchanged
        // Validates: Requirements 9.4
        #[test]
        fn prop_preview_cancellation_non_destructiveness(
            num_files in 1usize..10,
            file_contents in prop::collection::vec(
                prop::collection::vec(any::<u8>(), 0..1024),
                1..10
            )
        ) {
            // Ensure we have enough content for the files
            let num_files = num_files.min(file_contents.len());
            let file_contents = &file_contents[..num_files];
            
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            // Create test files with unique names and record their initial state
            let mut file_paths = Vec::new();
            let mut initial_file_states = Vec::new();
            
            for (i, content) in file_contents.iter().enumerate() {
                // Generate unique filename for each file
                let filename = format!("test_file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(content).unwrap();
                drop(file); // Ensure file is closed
                
                let path_str = file_path.to_string_lossy().to_string();
                file_paths.push(path_str.clone());
                
                // Record initial state
                let metadata = fs::metadata(&file_path).unwrap();
                initial_file_states.push((
                    path_str,
                    metadata.len(),
                    metadata.modified().unwrap(),
                    content.clone(),
                ));
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
            let file_ops_ref = file_ops.operations.clone();
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Step 1: Enable preview mode and process files
            processor.set_preview_mode(true);
            let preview_results = processor.process_files(file_paths.clone());
            
            // Verify preview results were generated
            prop_assert_eq!(
                preview_results.len(),
                num_files,
                "Preview must return results for all files"
            );
            
            // Step 2: Simulate cancellation by NOT disabling preview mode and NOT processing again
            // In a real application, the user would see the preview results and choose to cancel
            // This means we simply don't proceed with the actual operations
            
            // Property 18: After preview cancellation, all files must remain unchanged
            
            // 1. Verify NO actual file operations were performed during preview
            let operations = file_ops_ref.lock().unwrap();
            prop_assert_eq!(
                operations.len(),
                0,
                "Preview mode must not perform any file operations. Found {} operations",
                operations.len()
            );
            drop(operations);
            
            // 2. Verify all original files still exist with unchanged content
            for (original_path, original_size, _original_mod_time, original_content) in &initial_file_states {
                let path = std::path::Path::new(original_path);
                
                prop_assert!(
                    path.exists(),
                    "Original file must still exist after preview cancellation: {}",
                    original_path
                );
                
                // Verify file size hasn't changed
                let current_metadata = fs::metadata(path).unwrap();
                prop_assert_eq!(
                    current_metadata.len(),
                    *original_size,
                    "File size must not change after preview cancellation: {}",
                    original_path
                );
                
                // Verify file content hasn't changed
                let current_content = fs::read(path).unwrap();
                prop_assert_eq!(
                    &current_content,
                    original_content,
                    "File content must not change after preview cancellation: {}",
                    original_path
                );
            }
            
            // 3. Verify destination directory was NOT created
            let dest_dir = temp_dir.path().join("output");
            prop_assert!(
                !dest_dir.exists(),
                "Preview cancellation must not create destination directories"
            );
            
            // 4. Verify that if we were to disable preview mode and process again,
            // the operations would work (this confirms the preview was valid)
            // But we don't actually do this in the cancellation scenario
            
            // The key insight: cancellation means we simply don't proceed with actual operations
            // The preview mode already ensures non-destructiveness, and cancellation means
            // we stay in that non-destructive state
        }

        // Additional test: Verify preview cancellation with mixed file types and operations
        #[test]
        fn prop_preview_cancellation_with_mixed_operations(
            num_move_files in 1usize..5,
            num_copy_files in 1usize..5,
            file_content in prop::collection::vec(any::<u8>(), 0..512)
        ) {
            // Create temporary directory
            let temp_dir = TempDir::new().unwrap();
            
            let mut all_file_paths = Vec::new();
            let mut initial_file_states = Vec::new();
            
            // Create files that will match the "move" rule
            for i in 0..num_move_files {
                let filename = format!("move_file_{}.txt", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                drop(file);
                
                let path_str = file_path.to_string_lossy().to_string();
                all_file_paths.push(path_str.clone());
                
                let metadata = fs::metadata(&file_path).unwrap();
                initial_file_states.push((
                    path_str,
                    metadata.len(),
                    file_content.clone(),
                ));
            }
            
            // Create files that will match the "copy" rule
            for i in 0..num_copy_files {
                let filename = format!("copy_file_{}.jpg", i);
                let file_path = temp_dir.path().join(&filename);
                let mut file = fs::File::create(&file_path).unwrap();
                file.write_all(&file_content).unwrap();
                drop(file);
                
                let path_str = file_path.to_string_lossy().to_string();
                all_file_paths.push(path_str.clone());
                
                let metadata = fs::metadata(&file_path).unwrap();
                initial_file_states.push((
                    path_str,
                    metadata.len(),
                    file_content.clone(),
                ));
            }
            
            let total_files = all_file_paths.len();
            
            // Create rules for different operations
            let move_rule = Rule {
                id: "move-rule".to_string(),
                name: "Move Text Files".to_string(),
                priority: 1,
                conditions: vec![
                    Condition {
                        field: "extension".to_string(),
                        operator: "==".to_string(),
                        value: serde_json::json!("txt"),
                    },
                ],
                destination_pattern: temp_dir.path().join("moved").to_string_lossy().to_string(),
                operation: OperationType::Move,
            };
            
            let copy_rule = Rule {
                id: "copy-rule".to_string(),
                name: "Copy Image Files".to_string(),
                priority: 2,
                conditions: vec![
                    Condition {
                        field: "extension".to_string(),
                        operator: "==".to_string(),
                        value: serde_json::json!("jpg"),
                    },
                ],
                destination_pattern: temp_dir.path().join("copied").to_string_lossy().to_string(),
                operation: OperationType::Copy,
            };
            
            let rule_engine = RuleEngine::new(vec![move_rule, copy_rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = Box::new(MockFileOperations::new());
            let file_ops_ref = file_ops.operations.clone();
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                file_ops,
            );
            
            // Enable preview mode and process files
            processor.set_preview_mode(true);
            let preview_results = processor.process_files(all_file_paths.clone());
            
            // Verify preview results
            prop_assert_eq!(
                preview_results.len(),
                total_files,
                "Preview must return results for all files"
            );
            
            // Simulate cancellation - user decides not to proceed
            
            // Property 18: After preview cancellation with mixed operations, all files must remain unchanged
            
            // 1. Verify NO actual file operations were performed
            let operations = file_ops_ref.lock().unwrap();
            prop_assert_eq!(
                operations.len(),
                0,
                "Preview cancellation must not perform any file operations (move or copy). Found {} operations",
                operations.len()
            );
            drop(operations);
            
            // 2. Verify all original files still exist with unchanged content
            for (original_path, original_size, original_content) in &initial_file_states {
                let path = std::path::Path::new(original_path);
                
                prop_assert!(
                    path.exists(),
                    "Original file must still exist after preview cancellation: {}",
                    original_path
                );
                
                // Verify file size hasn't changed
                let current_metadata = fs::metadata(path).unwrap();
                prop_assert_eq!(
                    current_metadata.len(),
                    *original_size,
                    "File size must not change after preview cancellation: {}",
                    original_path
                );
                
                // Verify file content hasn't changed
                let current_content = fs::read(path).unwrap();
                prop_assert_eq!(
                    &current_content,
                    original_content,
                    "File content must not change after preview cancellation: {}",
                    original_path
                );
            }
            
            // 3. Verify destination directories were NOT created
            let moved_dir = temp_dir.path().join("moved");
            let copied_dir = temp_dir.path().join("copied");
            
            prop_assert!(
                !moved_dir.exists(),
                "Preview cancellation must not create 'moved' destination directory"
            );
            
            prop_assert!(
                !copied_dir.exists(),
                "Preview cancellation must not create 'copied' destination directory"
            );
            
            // 4. Verify preview results showed the correct intended operations
            let move_results: Vec<_> = preview_results.iter()
                .filter(|r| r.matched_rule.as_ref().map(|s| s.as_str()) == Some("Move Text Files"))
                .collect();
            
            let copy_results: Vec<_> = preview_results.iter()
                .filter(|r| r.matched_rule.as_ref().map(|s| s.as_str()) == Some("Copy Image Files"))
                .collect();
            
            prop_assert_eq!(
                move_results.len(),
                num_move_files,
                "Preview must show correct number of move operations"
            );
            
            prop_assert_eq!(
                copy_results.len(),
                num_copy_files,
                "Preview must show correct number of copy operations"
            );
            
            // All preview results should be successful
            for result in &preview_results {
                prop_assert!(
                    result.success,
                    "All preview results should be successful: {}",
                    result.source_path
                );
                
                prop_assert!(
                    result.destination_path.is_some(),
                    "All preview results should show destination: {}",
                    result.source_path
                );
            }
        }
    }
}
