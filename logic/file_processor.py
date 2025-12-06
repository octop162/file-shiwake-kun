import os
import shutil
import threading
from typing import List, Dict, Any, Tuple, Callable, Optional

from data.metadata_extractor import MetadataExtractor
from logic.rule_engine import RuleEngine
from data.file_operations import FileOperations

ProcessResult = Dict[str, Any] # 'status': 'success' | 'skipped' | 'failed'

class FileProcessor:
    """
    Orchestrates the file processing workflow, separating discovery from execution.
    """
    def __init__(self, config: Dict[str, Any], conflict_handler: callable = None):
        self.config = config
        self.metadata_extractor = MetadataExtractor()
        self.rule_engine = RuleEngine(config.get('rules', []))
        self.file_operations = FileOperations()
        self.conflict_handler = conflict_handler
        self.conflict_resolution_all = None # Can be 'skip', 'overwrite'

    def discover_operations(
        self, 
        file_paths: List[str], 
        rule_id: str,
        progress_callback: Optional[Callable[[int, int], None]] = None,
        cancel_check: Optional[Callable[[], bool]] = None
    ) -> List[Dict[str, Any]]:
        """
        Finds the operations that would be performed for a list of files
        based on a selected rule, but does not execute them.
        """
        # Pass the progress callback to the file discovery phase
        all_files_to_process = self._get_all_files(
            file_paths, 
            lambda count: progress_callback(count, 0) if progress_callback else None,
            cancel_check
        )
        operations = []
        
        selected_rule = next((r for r in self.rule_engine.rules if r['id'] == rule_id), None)
        if not selected_rule:
            return [{'file_path': fp, 'error': f"Rule with ID '{rule_id}' not found."} for fp in all_files_to_process]

        total_files = len(all_files_to_process)
        for i, file_path in enumerate(all_files_to_process):
            if cancel_check and cancel_check():
                raise InterruptedError("Operation cancelled by user.")

            if progress_callback:
                # 進捗をUIに通知 (現在のファイル番号, 総ファイル数)
                progress_callback(i + 1, total_files) # Now in analysis phase

            metadata = self.metadata_extractor.extract(file_path)
            if not metadata:
                operations.append({'file_path': file_path, 'error': "Failed to extract metadata."})
                continue

            if self.rule_engine._check_conditions(metadata, selected_rule.get('conditions', [])):
                dest_path = self.rule_engine._format_destination(metadata, selected_rule['destination_pattern'])
                operations.append({
                    'file_path': file_path,
                    'rule': selected_rule,
                    'dest_path': dest_path,
                    'error': None
                })
            else:
                operations.append({'file_path': file_path, 'error': "File did not match the selected rule."})
        
        return operations

    def _get_all_files(
        self, 
        paths: List[str], 
        progress_callback: Optional[Callable[[int], None]] = None,
        cancel_check: Optional[Callable[[], bool]] = None
    ) -> List[str]:
        # ... same as before
        all_files = []
        file_count = 0
        for path in paths:
            if cancel_check and cancel_check():
                raise InterruptedError("Operation cancelled by user.")

            if not os.path.exists(path):
                print(f"Warning: Path does not exist and will be skipped: {path}")
                continue

            if os.path.isdir(path):
                for root, _, files in os.walk(path):
                    if cancel_check and cancel_check():
                        raise InterruptedError("Operation cancelled by user.")

                    for name in files:
                        all_files.append(os.path.join(root, name))
                        file_count += 1
                        if progress_callback: progress_callback(file_count)
            else:
                all_files.append(path)
        return all_files

    def execute_operation(
        self, 
        file_path: str, 
        rule: Dict[str, Any], 
        dest_path: str, 
    ) -> ProcessResult:
        """
        Executes a single file operation based on a chosen rule.
        """
        result: ProcessResult = {
            'source_path': file_path, 'destination_path': dest_path, 'status': "failed",
            'matched_rule': rule['name'], 'error_message': None, 'operation': 'copy' # Always report copy
        }

        # Always use copy_file regardless of rule['operation'] to ensure safety
        file_op = self.file_operations.copy_file 
        # was: file_op = self.file_operations.copy_file if rule['operation'] == 'copy' else self.file_operations.move_file
        
        error = file_op(file_path, dest_path, overwrite=False)

        if error == "conflict":
            # If a global resolution is set, use it without asking the user
            if self.conflict_resolution_all:
                resolution = self.conflict_resolution_all
            elif self.conflict_handler:
                # Ask the user and potentially set the global resolution
                resolution_data = self.conflict_handler(file_path, dest_path)
                resolution = resolution_data.get("resolution")
                if resolution_data.get("apply_to_all") and resolution in ["overwrite", "skip"]:
                    self.conflict_resolution_all = resolution
            else:
                # No handler, default to error
                resolution = "error"

            if resolution == "skip":
                result['status'] = "skipped"
                return result
            elif resolution == "overwrite":
                error = file_op(file_path, dest_path, overwrite=True)
            elif resolution != "error": # This is a rename case
                new_dest_path = resolution
                result['destination_path'] = new_dest_path
                error = file_op(file_path, new_dest_path, overwrite=False)
        
        if error:
            result['error_message'] = error
        else:
            result['status'] = "success"
            
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
                "destination_pattern": dest_dir.replace('\\', '/') + "/{year}/{filename}{extension}",
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
