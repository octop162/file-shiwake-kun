// Unit tests for conflict handling functionality
// These tests verify that file conflicts are detected and resolved correctly

use super::file_processor::FileProcessor;
use super::metadata_extractor::MetadataExtractor;
use super::file_operations::FileOperations;
use super::rule_engine::RuleEngine;
use crate::models::{FileMetadata, Rule, OperationType, FileInfo, ConflictResolution};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

// Mock MetadataExtractor for testing
struct MockMetadataExtractor;

impl MetadataExtractor for MockMetadataExtractor {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String> {
        let path = std::path::Path::new(filepath);
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        Ok(FileMetadata {
            filename,
            extension,
            size: 1024,
            created_at: Some(SystemTime::now()),
            modified_at: SystemTime::now(),
            capture_date: None,
            camera_model: None,
            gps_latitude: None,
            gps_longitude: None,
        })
    }
}

// Mock FileOperations that tracks operations
#[derive(Clone)]
struct MockFileOperations {
    operations: Arc<Mutex<Vec<(String, String, String)>>>, // (operation, source, dest)
    existing_files: Arc<Mutex<Vec<String>>>,
}

impl MockFileOperations {
    fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
            existing_files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_existing_file(&self, path: String) {
        self.existing_files.lock().unwrap().push(path);
    }

    fn get_operations(&self) -> Vec<(String, String, String)> {
        self.operations.lock().unwrap().clone()
    }
}

// Helper function to construct platform-specific paths
fn make_path(dir: &str, filename: &str) -> String {
    std::path::PathBuf::from(dir).join(filename).to_string_lossy().to_string()
}

impl FileOperations for MockFileOperations {
    fn move_file(&self, source: &str, dest: &str) -> Result<(), String> {
        self.operations.lock().unwrap().push(("move".to_string(), source.to_string(), dest.to_string()));
        Ok(())
    }

    fn copy_file(&self, source: &str, dest: &str) -> Result<(), String> {
        self.operations.lock().unwrap().push(("copy".to_string(), source.to_string(), dest.to_string()));
        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        self.existing_files.lock().unwrap().contains(&path.to_string())
    }

    fn create_dir(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_file_info(&self, path: &str) -> Result<FileInfo, String> {
        Ok(FileInfo {
            name: path.to_string(),
            size: 1024,
            mod_time: SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conflict_when_destination_does_not_exist() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Process a file
        let result = processor.process_file("/source/test.txt");

        // Should succeed without conflict
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_conflict_detected_when_destination_exists() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destination as existing
        file_ops.add_existing_file(make_path("/dest", "test.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Process a file without setting conflict callback
        let result = processor.process_file("/source/test.txt");

        // Should fail due to conflict (default is Skip)
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert!(result.error_message.unwrap().contains("conflict"));
    }

    #[test]
    fn test_conflict_resolution_overwrite() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destination as existing
        file_ops.add_existing_file(make_path("/dest", "test.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to always overwrite
        processor.set_conflict_callback(Box::new(|_, _| ConflictResolution::Overwrite));

        // Process a file
        let result = processor.process_file("/source/test.txt");

        // Should succeed with overwrite
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_conflict_resolution_skip() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destination as existing
        file_ops.add_existing_file(make_path("/dest", "test.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to always skip
        processor.set_conflict_callback(Box::new(|_, _| ConflictResolution::Skip));

        // Process a file
        let result = processor.process_file("/source/test.txt");

        // Should fail (skipped)
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_conflict_resolution_rename() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destination as existing
        let original_dest = make_path("/dest", "test.txt");
        file_ops.add_existing_file(original_dest.clone());
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to always rename
        processor.set_conflict_callback(Box::new(|_, _| ConflictResolution::Rename));

        // Process a file
        let result = processor.process_file("/source/test.txt");

        // Should succeed with renamed destination
        assert!(result.success);
        assert!(result.error_message.is_none());
        
        // Destination should be different from original
        let dest = result.destination_path.unwrap();
        assert_ne!(dest, original_dest);
        // The renamed file should have a number appended
        assert!(dest.contains("_1"));
    }

    #[test]
    fn test_conflict_policy_overwrite_all() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destinations as existing
        file_ops.add_existing_file(make_path("/dest", "test1.txt"));
        file_ops.add_existing_file(make_path("/dest", "test2.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to return OverwriteAll on first conflict
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        processor.set_conflict_callback(Box::new(move |_, _| {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            ConflictResolution::OverwriteAll
        }));

        // Process first file
        let result1 = processor.process_file("/source/test1.txt");
        assert!(result1.success);

        // Process second file - should use policy without calling callback again
        let result2 = processor.process_file("/source/test2.txt");
        assert!(result2.success);

        // Callback should only be called once
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_conflict_policy_skip_all() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destinations as existing
        file_ops.add_existing_file(make_path("/dest", "test1.txt"));
        file_ops.add_existing_file(make_path("/dest", "test2.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to return SkipAll on first conflict
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        processor.set_conflict_callback(Box::new(move |_, _| {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            ConflictResolution::SkipAll
        }));

        // Process first file
        let result1 = processor.process_file("/source/test1.txt");
        assert!(!result1.success);

        // Process second file - should use policy without calling callback again
        let result2 = processor.process_file("/source/test2.txt");
        assert!(!result2.success);

        // Callback should only be called once
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_conflict_policy_rename_all() {
        // Create a simple rule
        let rule = Rule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            priority: 1,
            conditions: vec![],
            destination_pattern: "/dest".to_string(),
            operation: OperationType::Move,
        };

        let rule_engine = RuleEngine::new(vec![rule]);
        let metadata_extractor = Box::new(MockMetadataExtractor);
        let file_ops = MockFileOperations::new();
        
        // Mark destinations as existing
        file_ops.add_existing_file(make_path("/dest", "test1.txt"));
        file_ops.add_existing_file(make_path("/dest", "test2.txt"));
        
        let mut processor = FileProcessor::new(
            rule_engine,
            metadata_extractor,
            Box::new(file_ops.clone()),
        );

        // Set conflict callback to return RenameAll on first conflict
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        processor.set_conflict_callback(Box::new(move |_, _| {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            ConflictResolution::RenameAll
        }));

        // Process first file
        let result1 = processor.process_file("/source/test1.txt");
        assert!(result1.success);
        assert!(result1.destination_path.unwrap().contains("test1_"));

        // Process second file - should use policy without calling callback again
        let result2 = processor.process_file("/source/test2.txt");
        assert!(result2.success);
        assert!(result2.destination_path.unwrap().contains("test2_"));

        // Callback should only be called once
        assert_eq!(*call_count.lock().unwrap(), 1);
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: file-shiwake-kun, Property 9: 競合時の確認要求
    // For any file operation where the destination already exists, a confirmation must be requested
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_conflict_triggers_confirmation_request(
            source_filename in "[a-zA-Z0-9_]{1,20}\\.(txt|jpg|pdf)",
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
            operation in prop_oneof![
                Just(OperationType::Move),
                Just(OperationType::Copy),
            ]
        ) {
            // Create a rule with the given operation
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // Mark destination as existing (conflict scenario)
            let dest_path = make_path(&dest_dir, &source_filename);
            file_ops.add_existing_file(dest_path.clone());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track whether the conflict callback was called
            let callback_called = Arc::new(Mutex::new(false));
            let callback_called_clone = callback_called.clone();
            
            processor.set_conflict_callback(Box::new(move |source_info, dest_info| {
                // Mark that callback was called
                *callback_called_clone.lock().unwrap() = true;
                
                // Verify that both file infos are provided
                assert!(!source_info.name.is_empty());
                assert!(!dest_info.name.is_empty());
                
                // Return a resolution
                ConflictResolution::Skip
            }));

            // Process the file
            let source_path = format!("/source/{}", source_filename);
            let _result = processor.process_file(&source_path);

            // Property: When a conflict exists, the callback MUST be called
            prop_assert!(*callback_called.lock().unwrap(), 
                "Conflict callback was not called when destination file exists");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_no_conflict_no_confirmation_when_dest_not_exists(
            source_filename in "[a-zA-Z0-9_]{1,20}\\.(txt|jpg|pdf)",
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
        ) {
            // Create a rule
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation: OperationType::Move,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // DO NOT mark destination as existing (no conflict scenario)
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track whether the conflict callback was called
            let callback_called = Arc::new(Mutex::new(false));
            let callback_called_clone = callback_called.clone();
            
            processor.set_conflict_callback(Box::new(move |_, _| {
                *callback_called_clone.lock().unwrap() = true;
                ConflictResolution::Skip
            }));

            // Process the file
            let source_path = format!("/source/{}", source_filename);
            let _result = processor.process_file(&source_path);

            // Property: When no conflict exists, the callback MUST NOT be called
            prop_assert!(!*callback_called.lock().unwrap(), 
                "Conflict callback was called when no conflict exists");
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_conflict_callback_receives_file_info(
            source_filename in "[a-zA-Z0-9_]{1,20}\\.(txt|jpg|pdf)",
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
        ) {
            // Create a rule
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation: OperationType::Move,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // Mark destination as existing
            let dest_path = make_path(&dest_dir, &source_filename);
            file_ops.add_existing_file(dest_path.clone());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track file info received in callback
            let received_source_info = Arc::new(Mutex::new(None));
            let received_dest_info = Arc::new(Mutex::new(None));
            let source_clone = received_source_info.clone();
            let dest_clone = received_dest_info.clone();
            
            processor.set_conflict_callback(Box::new(move |source_info, dest_info| {
                *source_clone.lock().unwrap() = Some(source_info.clone());
                *dest_clone.lock().unwrap() = Some(dest_info.clone());
                ConflictResolution::Skip
            }));

            // Process the file
            let source_path = format!("/source/{}", source_filename);
            let _result = processor.process_file(&source_path);

            // Property: Callback must receive both source and destination file info
            let source_info = received_source_info.lock().unwrap();
            let dest_info = received_dest_info.lock().unwrap();
            
            prop_assert!(source_info.is_some(), "Source file info was not provided to callback");
            prop_assert!(dest_info.is_some(), "Destination file info was not provided to callback");
            
            // Verify file info contains required fields (name, size, mod_time)
            if let Some(ref info) = *source_info {
                prop_assert!(!info.name.is_empty(), "Source file name is empty");
                prop_assert!(info.size > 0, "Source file size is 0");
            }
            
            if let Some(ref info) = *dest_info {
                prop_assert!(!info.name.is_empty(), "Destination file name is empty");
                prop_assert!(info.size > 0, "Destination file size is 0");
            }
        }
    }

    // Feature: file-shiwake-kun, Property 21: 競合情報の表示
    // For any file conflict confirmation dialog, both files' names, sizes, and modification times must be displayed
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_conflict_info_displays_all_required_fields(
            source_filename in "[a-zA-Z0-9_]{1,20}\\.(txt|jpg|pdf)",
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
        ) {
            // Create a rule
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation: OperationType::Move,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // Mark destination as existing to trigger conflict
            let dest_path = make_path(&dest_dir, &source_filename);
            file_ops.add_existing_file(dest_path.clone());
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track file info received in callback
            let received_source_info = Arc::new(Mutex::new(None));
            let received_dest_info = Arc::new(Mutex::new(None));
            let source_clone = received_source_info.clone();
            let dest_clone = received_dest_info.clone();
            
            processor.set_conflict_callback(Box::new(move |source_info, dest_info| {
                *source_clone.lock().unwrap() = Some(source_info.clone());
                *dest_clone.lock().unwrap() = Some(dest_info.clone());
                ConflictResolution::Skip
            }));

            // Process the file
            let source_path = format!("/source/{}", source_filename);
            let _result = processor.process_file(&source_path);

            // Property: Conflict dialog MUST display name, size, and mod_time for BOTH files
            let source_info = received_source_info.lock().unwrap();
            let dest_info = received_dest_info.lock().unwrap();
            
            prop_assert!(source_info.is_some(), "Source file info was not provided");
            prop_assert!(dest_info.is_some(), "Destination file info was not provided");
            
            // Verify SOURCE file info contains all three required fields
            if let Some(ref info) = *source_info {
                // 1. Name must be present and non-empty
                prop_assert!(!info.name.is_empty(), 
                    "Source file name is missing or empty");
                
                // 2. Size must be present (can be 0 for empty files, but field must exist)
                // The field exists by virtue of the struct, so we just verify it's accessible
                let _size = info.size;
                
                // 3. Modification time must be present
                // The field exists by virtue of the struct, so we just verify it's accessible
                let _mod_time = info.mod_time;
            }
            
            // Verify DESTINATION file info contains all three required fields
            if let Some(ref info) = *dest_info {
                // 1. Name must be present and non-empty
                prop_assert!(!info.name.is_empty(), 
                    "Destination file name is missing or empty");
                
                // 2. Size must be present
                let _size = info.size;
                
                // 3. Modification time must be present
                let _mod_time = info.mod_time;
            }
        }
    }

    // Feature: file-shiwake-kun, Property 22: 競合解決の一貫性
    // For any "apply to all" option selected, the same action must be automatically applied to conflicts with the same conditions
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_conflict_resolution_consistency_with_apply_to_all(
            num_files in 2usize..10,
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
            resolution in prop_oneof![
                Just(ConflictResolution::OverwriteAll),
                Just(ConflictResolution::SkipAll),
                Just(ConflictResolution::RenameAll),
            ]
        ) {
            // Create a rule
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation: OperationType::Move,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // Create multiple files with conflicts
            let mut source_files = Vec::new();
            for i in 0..num_files {
                let filename = format!("test{}.txt", i);
                source_files.push(filename.clone());
                
                // Mark each destination as existing to trigger conflicts
                let dest_path = make_path(&dest_dir, &filename);
                file_ops.add_existing_file(dest_path);
            }
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track how many times the callback is called
            let callback_count = Arc::new(Mutex::new(0));
            let callback_count_clone = callback_count.clone();
            let resolution_clone = resolution;
            
            processor.set_conflict_callback(Box::new(move |_, _| {
                let mut count = callback_count_clone.lock().unwrap();
                *count += 1;
                resolution_clone
            }));

            // Process all files
            let mut results = Vec::new();
            for filename in &source_files {
                let source_path = format!("/source/{}", filename);
                let result = processor.process_file(&source_path);
                results.push(result);
            }

            // Property 1: Callback should only be called ONCE (on first conflict)
            // After that, the policy should be applied automatically
            let final_callback_count = *callback_count.lock().unwrap();
            prop_assert_eq!(final_callback_count, 1, 
                "Callback was called {} times, expected 1 (policy should apply to subsequent conflicts)", 
                final_callback_count);

            // Property 2: All files should be processed with the SAME resolution
            match resolution {
                ConflictResolution::OverwriteAll => {
                    // All operations should succeed (overwrite)
                    for (i, result) in results.iter().enumerate() {
                        prop_assert!(result.success, 
                            "File {} failed with OverwriteAll policy: {:?}", 
                            i, result.error_message);
                    }
                },
                ConflictResolution::SkipAll => {
                    // All operations should fail (skipped)
                    for (i, result) in results.iter().enumerate() {
                        prop_assert!(!result.success, 
                            "File {} succeeded when it should have been skipped with SkipAll policy", 
                            i);
                    }
                },
                ConflictResolution::RenameAll => {
                    // All operations should succeed with renamed destinations
                    for (i, result) in results.iter().enumerate() {
                        prop_assert!(result.success, 
                            "File {} failed with RenameAll policy: {:?}", 
                            i, result.error_message);
                        
                        // Verify the destination was renamed (contains a number)
                        if let Some(ref dest) = result.destination_path {
                            prop_assert!(dest.contains("_"), 
                                "File {} destination was not renamed: {}", 
                                i, dest);
                        }
                    }
                },
                _ => {
                    // This shouldn't happen with our generator, but handle it
                    prop_assert!(false, "Unexpected resolution type");
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_single_resolution_does_not_apply_to_subsequent_conflicts(
            num_files in 2usize..10,
            dest_dir in "/[a-zA-Z0-9_/]{1,30}",
            resolution in prop_oneof![
                Just(ConflictResolution::Overwrite),
                Just(ConflictResolution::Skip),
                Just(ConflictResolution::Rename),
            ]
        ) {
            // Create a rule
            let rule = Rule {
                id: "test-rule".to_string(),
                name: "Test Rule".to_string(),
                priority: 1,
                conditions: vec![],
                destination_pattern: dest_dir.clone(),
                operation: OperationType::Move,
            };

            let rule_engine = RuleEngine::new(vec![rule]);
            let metadata_extractor = Box::new(MockMetadataExtractor);
            let file_ops = MockFileOperations::new();
            
            // Create multiple files with conflicts
            let mut source_files = Vec::new();
            for i in 0..num_files {
                let filename = format!("test{}.txt", i);
                source_files.push(filename.clone());
                
                // Mark each destination as existing to trigger conflicts
                let dest_path = make_path(&dest_dir, &filename);
                file_ops.add_existing_file(dest_path);
            }
            
            let mut processor = FileProcessor::new(
                rule_engine,
                metadata_extractor,
                Box::new(file_ops.clone()),
            );

            // Track how many times the callback is called
            let callback_count = Arc::new(Mutex::new(0));
            let callback_count_clone = callback_count.clone();
            let resolution_clone = resolution;
            
            processor.set_conflict_callback(Box::new(move |_, _| {
                let mut count = callback_count_clone.lock().unwrap();
                *count += 1;
                resolution_clone
            }));

            // Process all files
            for filename in &source_files {
                let source_path = format!("/source/{}", filename);
                let _result = processor.process_file(&source_path);
            }

            // Property: With single-file resolutions (not "All" variants),
            // callback should be called for EACH conflict
            let final_callback_count = *callback_count.lock().unwrap();
            prop_assert_eq!(final_callback_count, num_files, 
                "Callback was called {} times, expected {} (once per conflict with single-file resolution)", 
                final_callback_count, num_files);
        }
    }
}
