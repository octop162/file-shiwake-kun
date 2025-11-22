import os
import shutil
from pathlib import Path
from typing import Optional

class FileOperations:
    """
    A collection of static methods for safe file system operations.
    """

    @staticmethod
    def _ensure_parent_dir(dest_path: str) -> None:
        """
        Ensures the parent directory of the destination path exists.
        Creates it if it does not.
        """
        parent_dir = Path(dest_path).parent
        os.makedirs(parent_dir, exist_ok=True)

    @staticmethod
    def move_file(source: str, dest: str, overwrite: bool = False) -> Optional[str]:
        """
        Safely moves a file from source to dest.
        - If overwrite is False, checks for existence and returns "conflict".
        - Returns an error message string if it fails, otherwise None.
        """
        try:
            if not os.path.exists(source):
                return f"Source file does not exist: {source}"
            if not overwrite and os.path.exists(dest):
                return "conflict"
            FileOperations._ensure_parent_dir(dest)
            shutil.move(source, dest)
            return None
        except Exception as e:
            return f"Failed to move '{source}' to '{dest}': {e}"

    @staticmethod
    def copy_file(source: str, dest: str, overwrite: bool = False) -> Optional[str]:
        """
        Safely copies a file from source to dest, preserving metadata.
        - If overwrite is False, checks for existence and returns "conflict".
        - Returns an error message string if it fails, otherwise None.
        """
        try:
            if not os.path.exists(source):
                return f"Source file does not exist: {source}"
            if not overwrite and os.path.exists(dest):
                return "conflict"
            FileOperations._ensure_parent_dir(dest)
            shutil.copy2(source, dest)
            return None
        except Exception as e:
            return f"Failed to copy '{source}' to '{dest}': {e}"

if __name__ == '__main__':
    # This block is for demonstration and basic testing.
    print("--- Initializing File Operations Test ---")

    # Create a test directory and a dummy file
    test_dir = 'test_op_dir'
    source_file = os.path.join(test_dir, 'source.txt')
    
    os.makedirs(test_dir, exist_ok=True)
    with open(source_file, 'w') as f:
        f.write("This is a test file for file operations.")
    
    print(f"Created source file: {source_file}")

    # --- Test 1: Copy Operation ---
    print("\n--- Test 1: Copy Operation ---")
    copy_dest_dir = os.path.join(test_dir, 'copy', 'nested')
    copy_dest_file = os.path.join(copy_dest_dir, 'copied.txt')
    
    print(f"Attempting to copy to: {copy_dest_file}")
    error = FileOperations.copy_file(source_file, copy_dest_file)
    
    assert error is None
    assert os.path.exists(source_file)
    assert os.path.exists(copy_dest_file)
    print("Copy operation successful.")
    
    # --- Test 2: Move Operation ---
    print("\n--- Test 2: Move Operation ---")
    move_dest_dir = os.path.join(test_dir, 'move', 'nested')
    move_dest_file = os.path.join(move_dest_dir, 'moved.txt')

    print(f"Attempting to move {copy_dest_file} to: {move_dest_file}")
    error = FileOperations.move_file(copy_dest_file, move_dest_file)
    
    assert error is None
    assert not os.path.exists(copy_dest_file) # Original should be gone
    assert os.path.exists(move_dest_file)
    print("Move operation successful.")
    
    # --- Test 3: Failure case (source doesn't exist) ---
    print("\n--- Test 3: Failure Case ---")
    non_existent_source = os.path.join(test_dir, 'ghost.txt')
    error = FileOperations.copy_file(non_existent_source, move_dest_file)
    
    assert error is not None
    print(f"Received expected error: {error}")

    # Clean up the test directory
    shutil.rmtree(test_dir)
    print(f"\n--- Test complete. Cleaned up {test_dir} ---")
