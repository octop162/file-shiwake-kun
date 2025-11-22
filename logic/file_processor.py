import os
import shutil
from typing import List, Dict, Any

# Assuming the other modules are in the same project structure
from data.metadata_extractor import MetadataExtractor
from logic.rule_engine import RuleEngine
from data.file_operations import FileOperations
from data.config_manager import ConfigManager

# Define a type hint for the result dictionary for clarity
ProcessResult = Dict[str, Any]

class FileProcessor:
    """
    Orchestrates the entire file processing workflow.
    """
    def __init__(self, config: Dict[str, Any], conflict_handler: callable = None):
        """
        Initializes the FileProcessor.

        Args:
            config: The application configuration dictionary.
            conflict_handler: A callable that resolves file conflicts.
                              It should take (source, dest) and return
                              'overwrite', 'skip', or a new destination path.
        """
        self.config = config
        self.metadata_extractor = MetadataExtractor()
        self.rule_engine = RuleEngine(config.get('rules', []))
        self.file_operations = FileOperations()
        self.conflict_handler = conflict_handler

    def process_files(self, file_paths: List[str]) -> List[ProcessResult]:
        """
        Processes a list of file or directory paths.

        Args:
            file_paths: A list of absolute paths to files or directories.

        Returns:
            A list of dictionaries, where each dictionary represents the result
            of processing a single file.
        """
        all_files_to_process = self._get_all_files(file_paths)
        results = []

        for file_path in all_files_to_process:
            result = self._process_single_file(file_path)
            results.append(result)
            
        return results

    def _get_all_files(self, paths: List[str]) -> List[str]:
        """
        Expands a list of paths to include all files within any directories.
        """
        all_files = []
        for path in paths:
            if not os.path.exists(path):
                print(f"Warning: Path does not exist and will be skipped: {path}")
                continue
            
            if os.path.isdir(path):
                for root, _, files in os.walk(path):
                    for name in files:
                        all_files.append(os.path.join(root, name))
            else:
                all_files.append(path)
        return all_files

    def _process_single_file(self, file_path: str) -> ProcessResult:
        """
        Processes a single file.
        """
        result: ProcessResult = {
            'source_path': file_path,
            'destination_path': None,
            'success': False,
            'matched_rule': None,
            'error_message': None,
            'operation': None
        }

        # 1. Extract metadata
        metadata = self.metadata_extractor.extract(file_path)
        if not metadata:
            result['error_message'] = "Failed to extract metadata."
            return result

        # 2. Find matching rule
        rule, dest_path = self.rule_engine.process_file(metadata)

        if not rule or not dest_path:
            result['error_message'] = "No matching rule found."
            result['success'] = True 
            return result

        result['destination_path'] = dest_path
        result['matched_rule'] = rule['name']
        result['operation'] = rule['operation']
        
        # 3. Handle preview mode
        if self.config.get('preview_mode', False):
            result['success'] = True 
            return result

        # 4. Perform file operation
        file_op = self.file_operations.copy_file if rule['operation'] == 'copy' else self.file_operations.move_file
        
        error = file_op(file_path, dest_path, overwrite=False)
        
        if error == "conflict":
            if self.conflict_handler:
                resolution = self.conflict_handler(file_path, dest_path)
                if resolution == "skip":
                    result['error_message'] = "Operation skipped by user due to conflict."
                    result['success'] = True # Skipping is a "successful" outcome in this context
                    return result
                elif resolution == "overwrite":
                    error = file_op(file_path, dest_path, overwrite=True)
                else: # Assumes a new path was returned
                    new_dest_path = resolution
                    result['destination_path'] = new_dest_path
                    error = file_op(file_path, new_dest_path, overwrite=False) # Check again for safety
            else:
                # Default behavior if no handler is provided
                error = "Destination file exists (conflict)."
        
        if error:
            result['error_message'] = error
        else:
            result['success'] = True
            
        return result


if __name__ == '__main__':
    # This block demonstrates the end-to-end logic of Phase 1.
    print("--- Initializing File Processor Test ---")
    
    # 1. Setup a test environment
    test_root = 'processor_test_env'
    source_dir = os.path.join(test_root, 'source')
    dest_dir = os.path.join(test_root, 'destination')
    
    # Clean up previous run
    if os.path.exists(test_root):
        shutil.rmtree(test_root)
        
    os.makedirs(source_dir)
    
    # Create a dummy file
    test_file = os.path.join(source_dir, 'test_image.jpg')
    with open(test_file, 'w') as f:
        f.write("dummy image content")
        
    print(f"Created test file: {test_file}")
    
    # 2. Setup a test configuration
    test_config = {
        "preview_mode": False,
        "rules": [
            {
                "id": "rule-jpg",
                "name": "Sort JPGs",
                "priority": 1,
                "operation": "move",
                "destination_pattern": os.path.join(dest_dir, "{year}", "{filename}{extension}"),
                "conditions": [{"field": "extension", "operator": "==", "value": ".jpg"}]
            }
        ]
    }
    
    # 3. Run the processor
    processor = FileProcessor(test_config)
    results = processor.process_files([source_dir]) # Process the whole directory
    
    # 4. Assert the results
    print("\n--- Processing Results ---")
    print(results)
    
    assert len(results) == 1
    result = results[0]
    
    assert result['success'] is True
    assert result['error_message'] is None
    assert result['matched_rule'] == "Sort JPGs"
    
    # Get current year for destination path assertion
    year = datetime.date.today().year
    expected_dest = os.path.join(dest_dir, str(year), 'test_image.jpg')
    
    # Use os.path.normpath to handle path separators
    assert os.path.normpath(result['destination_path']) == os.path.normpath(expected_dest)
    
    # Check that the file was actually moved
    assert not os.path.exists(test_file)
    assert os.path.exists(expected_dest)
    
    print("\nFile processor test successful. File was moved correctly.")
    
    # Clean up
    shutil.rmtree(test_root)
    print(f"--- Test complete. Cleaned up {test_root} ---")
