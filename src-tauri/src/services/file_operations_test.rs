use super::file_operations::{DefaultFileOperations, FileOperations};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // Create source file
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"test content").unwrap();

        let ops = DefaultFileOperations;
        let result = ops.copy_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(source_path.exists());
        assert!(dest_path.exists());

        let source_content = fs::read_to_string(&source_path).unwrap();
        let dest_content = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(source_content, dest_content);
    }

    #[test]
    fn test_move_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        // Create source file
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"test content").unwrap();

        let ops = DefaultFileOperations;
        let result = ops.move_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(!source_path.exists());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_copy_file_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("subdir").join("dest.txt");

        // Create source file
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"test content").unwrap();

        let ops = DefaultFileOperations;
        let result = ops.copy_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(dest_path.parent().unwrap().exists());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_move_file_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("subdir").join("dest.txt");

        // Create source file
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"test content").unwrap();

        let ops = DefaultFileOperations;
        let result = ops.move_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert!(dest_path.parent().unwrap().exists());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_copy_file_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("nonexistent.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        let ops = DefaultFileOperations;
        let result = ops.copy_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_move_file_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("nonexistent.txt");
        let dest_path = temp_dir.path().join("dest.txt");

        let ops = DefaultFileOperations;
        let result = ops.move_file(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let ops = DefaultFileOperations;
        assert!(!ops.exists(file_path.to_str().unwrap()));

        fs::File::create(&file_path).unwrap();
        assert!(ops.exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_create_dir() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir").join("nested");

        let ops = DefaultFileOperations;
        let result = ops.create_dir(dir_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_get_file_info() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let ops = DefaultFileOperations;
        let result = ops.get_file_info(file_path.to_str().unwrap());

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "test.txt");
        assert_eq!(info.size, 12);
    }

    #[test]
    fn test_get_file_info_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let ops = DefaultFileOperations;
        let result = ops.get_file_info(file_path.to_str().unwrap());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_path_validation_null_character() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        
        // Create source file
        let mut file = fs::File::create(&source_path).unwrap();
        file.write_all(b"test").unwrap();

        let ops = DefaultFileOperations;
        
        // Try to create directory with null character
        let result = ops.create_dir("test\0dir");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null character"));
    }
}

// Feature: file-shiwake-kun, Property 7: 移動操作の完全性
// For any file, when a move operation succeeds, the file must not exist at the original location and must exist at the destination location
// Validates: Requirements 4.2
#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_move_operation_completeness(
            filename in "[a-zA-Z0-9_-]{1,20}\\.(txt|jpg|png|pdf)",
            content in prop::collection::vec(any::<u8>(), 0..1024),
            dest_subdir in prop::option::of("[a-zA-Z0-9_-]{1,10}")
        ) {
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join(&filename);
            
            // Create destination path (optionally in a subdirectory)
            let dest_path = if let Some(subdir) = dest_subdir {
                temp_dir.path().join(subdir).join(&filename)
            } else {
                temp_dir.path().join(format!("moved_{}", filename))
            };
            
            // Create source file with content
            let mut file = fs::File::create(&source_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // Ensure file is closed
            
            // Verify source exists before move
            prop_assert!(source_path.exists(), "Source file should exist before move");
            
            // Perform move operation
            let ops = DefaultFileOperations;
            let result = ops.move_file(
                source_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
            );
            
            // Property 7: Move operation completeness
            // When move succeeds, source must not exist and destination must exist
            if result.is_ok() {
                prop_assert!(
                    !source_path.exists(),
                    "After successful move, source file must not exist at original location: {:?}",
                    source_path
                );
                prop_assert!(
                    dest_path.exists(),
                    "After successful move, file must exist at destination: {:?}",
                    dest_path
                );
                
                // Verify content integrity
                let dest_content = fs::read(&dest_path).unwrap();
                prop_assert_eq!(
                    content,
                    dest_content,
                    "File content must be preserved after move"
                );
            }
        }
        
        // Feature: file-shiwake-kun, Property 8: コピー操作の完全性
        // For any file, when a copy operation succeeds, the file must exist at both the original location and the destination location
        // Validates: Requirements 4.3
        #[test]
        fn prop_copy_operation_completeness(
            filename in "[a-zA-Z0-9_-]{1,20}\\.(txt|jpg|png|pdf)",
            content in prop::collection::vec(any::<u8>(), 0..1024),
            dest_subdir in prop::option::of("[a-zA-Z0-9_-]{1,10}")
        ) {
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join(&filename);
            
            // Create destination path (optionally in a subdirectory)
            let dest_path = if let Some(subdir) = dest_subdir {
                temp_dir.path().join(subdir).join(&filename)
            } else {
                temp_dir.path().join(format!("copied_{}", filename))
            };
            
            // Create source file with content
            let mut file = fs::File::create(&source_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // Ensure file is closed
            
            // Verify source exists before copy
            prop_assert!(source_path.exists(), "Source file should exist before copy");
            
            // Perform copy operation
            let ops = DefaultFileOperations;
            let result = ops.copy_file(
                source_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
            );
            
            // Property 8: Copy operation completeness
            // When copy succeeds, both source and destination must exist
            if result.is_ok() {
                prop_assert!(
                    source_path.exists(),
                    "After successful copy, source file must still exist at original location: {:?}",
                    source_path
                );
                prop_assert!(
                    dest_path.exists(),
                    "After successful copy, file must exist at destination: {:?}",
                    dest_path
                );
                
                // Verify content integrity - both files should have the same content
                let source_content = fs::read(&source_path).unwrap();
                let dest_content = fs::read(&dest_path).unwrap();
                
                // First verify original content is preserved in source
                prop_assert_eq!(
                    &content,
                    &source_content,
                    "Source file content must be preserved after copy"
                );
                
                // Then verify source and destination have identical content
                prop_assert_eq!(
                    &source_content,
                    &dest_content,
                    "Source and destination files must have identical content after copy"
                );
            }
        }
        
        // Feature: file-shiwake-kun, Property 19: ディレクトリの自動作成
        // For any destination path that doesn't exist, all necessary parent directories must be created
        // Validates: Requirements 10.1
        #[test]
        fn prop_automatic_directory_creation(
            filename in "[a-zA-Z0-9_-]{1,20}\\.(txt|jpg|png|pdf)",
            content in prop::collection::vec(any::<u8>(), 0..1024),
            // Generate nested directory structure (1-4 levels deep)
            dir_levels in prop::collection::vec("[a-zA-Z0-9_-]{1,10}", 1..5)
        ) {
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join(&filename);
            
            // Build nested destination path
            let mut dest_path = temp_dir.path().to_path_buf();
            for dir in &dir_levels {
                dest_path = dest_path.join(dir);
            }
            dest_path = dest_path.join(&filename);
            
            // Verify that the destination directory structure does NOT exist yet
            if let Some(parent) = dest_path.parent() {
                prop_assert!(
                    !parent.exists(),
                    "Parent directory should not exist before operation: {:?}",
                    parent
                );
            }
            
            // Create source file with content
            let mut file = fs::File::create(&source_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // Ensure file is closed
            
            // Perform copy operation (which should create all parent directories)
            let ops = DefaultFileOperations;
            let result = ops.copy_file(
                source_path.to_str().unwrap(),
                dest_path.to_str().unwrap(),
            );
            
            // Property 19: Automatic directory creation
            // When operation succeeds, all parent directories must have been created
            if result.is_ok() {
                // Verify the destination file exists
                prop_assert!(
                    dest_path.exists(),
                    "Destination file must exist after successful operation: {:?}",
                    dest_path
                );
                
                // Verify all parent directories were created
                if let Some(parent) = dest_path.parent() {
                    prop_assert!(
                        parent.exists(),
                        "All parent directories must be created: {:?}",
                        parent
                    );
                    prop_assert!(
                        parent.is_dir(),
                        "Parent path must be a directory: {:?}",
                        parent
                    );
                }
                
                // Verify each level of the directory hierarchy exists
                let mut current_path = temp_dir.path().to_path_buf();
                for dir in &dir_levels {
                    current_path = current_path.join(dir);
                    prop_assert!(
                        current_path.exists(),
                        "Directory level must exist: {:?}",
                        current_path
                    );
                    prop_assert!(
                        current_path.is_dir(),
                        "Path must be a directory: {:?}",
                        current_path
                    );
                }
                
                // Verify file content is correct
                let dest_content = fs::read(&dest_path).unwrap();
                prop_assert_eq!(
                    &content,
                    &dest_content,
                    "File content must be preserved after copy with directory creation"
                );
            }
        }
        
        // Feature: file-shiwake-kun, Property 20: 移動先パスの検証
        // For any destination path, it must be validated before attempting file operations
        // Validates: Requirements 10.5
        #[test]
        fn prop_destination_path_validation(
            filename in "[a-zA-Z0-9_-]{1,20}\\.(txt|jpg|png|pdf)",
            content in prop::collection::vec(any::<u8>(), 0..1024)
        ) {
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join(&filename);
            
            // Create source file with content
            let mut file = fs::File::create(&source_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // Ensure file is closed
            
            let ops = DefaultFileOperations;
            
            // Property 20: Path validation before file operations
            // Test 1: Paths with null characters should be rejected
            let invalid_dest_null = format!("test\0dir");
            
            // Try copy operation with null character in path
            let copy_result = ops.copy_file(
                source_path.to_str().unwrap(),
                &invalid_dest_null,
            );
            
            // Must fail with validation error before attempting operation
            prop_assert!(
                copy_result.is_err(),
                "Copy operation must reject path with null character"
            );
            
            // Verify the error message indicates validation failure
            if let Err(err_msg) = &copy_result {
                prop_assert!(
                    err_msg.contains("null character") || 
                    err_msg.contains("Invalid") ||
                    err_msg.contains("Failed to"),
                    "Error message should indicate validation failure: {}",
                    err_msg
                );
            }
            
            // Try move operation with null character in path
            let move_result = ops.move_file(
                source_path.to_str().unwrap(),
                &invalid_dest_null,
            );
            
            // Must fail with validation error before attempting operation
            prop_assert!(
                move_result.is_err(),
                "Move operation must reject path with null character"
            );
            
            // Verify source file still exists (operation was rejected before execution)
            prop_assert!(
                source_path.exists(),
                "Source file must still exist after rejected move operation"
            );
            
            // Test 2: Valid paths should pass validation
            let valid_dest = temp_dir.path().join(format!("valid_{}", filename));
            
            let copy_result = ops.copy_file(
                source_path.to_str().unwrap(),
                valid_dest.to_str().unwrap(),
            );
            
            // Valid paths should succeed (or fail for reasons other than validation)
            if copy_result.is_err() {
                let err_msg = copy_result.unwrap_err();
                // Should not fail due to path validation
                prop_assert!(
                    !err_msg.contains("null character") && !err_msg.contains("Invalid path"),
                    "Valid path should not fail validation: {}",
                    err_msg
                );
            } else {
                // If copy succeeded, verify the file exists at destination
                prop_assert!(
                    valid_dest.exists(),
                    "Destination file must exist after successful copy"
                );
            }
            
            // Test 3: create_dir should also validate paths with null characters
            let invalid_dir_null = format!("test\0dir");
            let create_result = ops.create_dir(&invalid_dir_null);
            
            prop_assert!(
                create_result.is_err(),
                "create_dir must reject path with null character"
            );
            
            if let Err(err_msg) = create_result {
                prop_assert!(
                    err_msg.contains("null character") || err_msg.contains("Invalid"),
                    "Error message should indicate validation failure: {}",
                    err_msg
                );
            }
        }
        
        // Feature: file-shiwake-kun, Property 13: クロスプラットフォームパス処理
        // For any file path, platform-specific path separators must be handled correctly
        // Validates: Requirements 7.4
        #[test]
        fn prop_cross_platform_path_handling(
            filename in "[a-zA-Z0-9_-]{1,20}\\.(txt|jpg|png|pdf)",
            content in prop::collection::vec(any::<u8>(), 0..1024),
            dir_components in prop::collection::vec("[a-zA-Z0-9_-]{1,10}", 1..4)
        ) {
            use std::path::MAIN_SEPARATOR;
            
            // Create a temporary directory for the test
            let temp_dir = TempDir::new().unwrap();
            let source_path = temp_dir.path().join(&filename);
            
            // Create source file with content
            let mut file = fs::File::create(&source_path).unwrap();
            file.write_all(&content).unwrap();
            drop(file); // Ensure file is closed
            
            let ops = DefaultFileOperations;
            
            // Property 13: Cross-platform path handling
            // Test that paths work correctly regardless of separator used
            
            // Build a path using the platform's native separator (PathBuf handles this)
            let mut native_dest = temp_dir.path().to_path_buf();
            for component in &dir_components {
                native_dest = native_dest.join(component);
            }
            native_dest = native_dest.join(&filename);
            
            // Test 1: Native path separator should work
            let result_native = ops.copy_file(
                source_path.to_str().unwrap(),
                native_dest.to_str().unwrap(),
            );
            
            prop_assert!(
                result_native.is_ok(),
                "Copy operation must succeed with native path separators: {:?}",
                result_native
            );
            
            prop_assert!(
                native_dest.exists(),
                "File must exist at destination with native path: {:?}",
                native_dest
            );
            
            // Verify content integrity
            let dest_content = fs::read(&native_dest).unwrap();
            prop_assert_eq!(
                &content,
                &dest_content,
                "File content must be preserved with native path separators"
            );
            
            // Test 2: Build path string manually with platform separator
            let manual_path_str = {
                let mut path_str = temp_dir.path().to_string_lossy().to_string();
                for component in &dir_components {
                    path_str.push(MAIN_SEPARATOR);
                    path_str.push_str(component);
                }
                path_str.push(MAIN_SEPARATOR);
                path_str.push_str(&format!("manual_{}", filename));
                path_str
            };
            
            let result_manual = ops.copy_file(
                source_path.to_str().unwrap(),
                &manual_path_str,
            );
            
            prop_assert!(
                result_manual.is_ok(),
                "Copy operation must succeed with manually constructed platform-specific path: {:?}",
                result_manual
            );
            
            let manual_dest = std::path::PathBuf::from(&manual_path_str);
            prop_assert!(
                manual_dest.exists(),
                "File must exist at destination with manually constructed path: {:?}",
                manual_dest
            );
            
            // Test 3: PathBuf normalization - paths with redundant separators should work
            // Create a path with double separators (which should be normalized)
            let redundant_path_str = {
                let mut path_str = temp_dir.path().to_string_lossy().to_string();
                for component in &dir_components {
                    // Add double separator intentionally
                    path_str.push(MAIN_SEPARATOR);
                    path_str.push(MAIN_SEPARATOR);
                    path_str.push_str(component);
                }
                path_str.push(MAIN_SEPARATOR);
                path_str.push_str(&format!("redundant_{}", filename));
                path_str
            };
            
            let result_redundant = ops.copy_file(
                source_path.to_str().unwrap(),
                &redundant_path_str,
            );
            
            // PathBuf should normalize redundant separators
            prop_assert!(
                result_redundant.is_ok(),
                "Copy operation must handle paths with redundant separators: {:?}",
                result_redundant
            );
            
            let redundant_dest = std::path::PathBuf::from(&redundant_path_str);
            prop_assert!(
                redundant_dest.exists(),
                "File must exist even with redundant separators in path: {:?}",
                redundant_dest
            );
            
            // Test 4: Verify that create_dir also handles platform-specific separators
            let dir_path_str = {
                let mut path_str = temp_dir.path().to_string_lossy().to_string();
                path_str.push(MAIN_SEPARATOR);
                path_str.push_str("test_dir");
                path_str.push(MAIN_SEPARATOR);
                path_str.push_str("nested");
                path_str
            };
            
            let create_result = ops.create_dir(&dir_path_str);
            
            prop_assert!(
                create_result.is_ok(),
                "create_dir must handle platform-specific path separators: {:?}",
                create_result
            );
            
            let created_dir = std::path::PathBuf::from(&dir_path_str);
            prop_assert!(
                created_dir.exists() && created_dir.is_dir(),
                "Directory must be created with platform-specific separators: {:?}",
                created_dir
            );
            
            // Test 5: Verify exists() works with platform-specific paths
            prop_assert!(
                ops.exists(native_dest.to_str().unwrap()),
                "exists() must work with native platform paths"
            );
            
            prop_assert!(
                ops.exists(&manual_path_str),
                "exists() must work with manually constructed platform-specific paths"
            );
        }
    }
}
