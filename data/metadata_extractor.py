import os
import datetime
from PIL import Image
from PIL.ExifTags import TAGS
from typing import Dict, Any, Optional

class MetadataExtractor:
    """
    Extracts filesystem and EXIF metadata from a file.
    """

    def extract(self, file_path: str) -> Optional[Dict[str, Any]]:
        """
        Extracts metadata from the given file.

        Args:
            file_path: The absolute path to the file.

        Returns:
            A dictionary containing the extracted metadata, or None if the file doesn't exist.
        """
        if not os.path.exists(file_path):
            return None

        try:
            # 1. Filesystem metadata
            stat_info = os.stat(file_path)
            metadata = {
                'filename': os.path.basename(file_path),
                'extension': os.path.splitext(file_path)[1].lower(),
                'size': stat_info.st_size,
                'created_at': datetime.datetime.fromtimestamp(stat_info.st_ctime),
                'modified_at': datetime.datetime.fromtimestamp(stat_info.st_mtime),
                'capture_date': None,
                'camera_model': None,
                'gps_info': None,
            }

            # 2. Image (EXIF) metadata for common image types
            image_extensions = ['.jpg', '.jpeg', '.tiff', '.heic', '.png']
            if metadata['extension'] in image_extensions:
                metadata.update(self._extract_exif(file_path))

            # Use modification time as fallback for capture date if not present
            if metadata['capture_date'] is None:
                metadata['capture_date'] = metadata['modified_at']

            return metadata

        except Exception as e:
            print(f"Error extracting metadata for {file_path}: {e}")
            return None

    def _extract_exif(self, file_path: str) -> Dict[str, Any]:
        """
        Private method to extract EXIF data from an image file.
        """
        exif_data = {}
        try:
            with Image.open(file_path) as img:
                exif = img._getexif()
                if exif:
                    for tag, value in exif.items():
                        tag_name = TAGS.get(tag, tag)
                        exif_data[tag_name] = value

            # Extract specific, commonly used fields
            capture_date = exif_data.get('DateTimeOriginal') or exif_data.get('DateTimeDigitized')
            camera_model = exif_data.get('Model')
            
            # Parse capture date string to datetime object
            parsed_capture_date = None
            if capture_date and isinstance(capture_date, str):
                try:
                    # Common EXIF date format: "YYYY:MM:DD HH:MM:SS"
                    parsed_capture_date = datetime.datetime.strptime(capture_date, '%Y:%m:%d %H:%M:%S')
                except ValueError:
                    print(f"Could not parse date: {capture_date}")


            return {
                'capture_date': parsed_capture_date,
                'camera_model': camera_model.strip() if camera_model else None
            }

        except Exception as e:
            # Pillow might fail on some files, which is fine.
            # print(f"Could not extract EXIF from {file_path}: {e}")
            return {}


if __name__ == '__main__':
    # This block is for demonstration and basic testing.
    # A proper unit test would use mock files.

    # Create a dummy file for testing filesystem metadata
    test_dir = 'test_files'
    if not os.path.exists(test_dir):
        os.makedirs(test_dir)
    
    test_file_path = os.path.join(test_dir, 'test_file.txt')
    with open(test_file_path, 'w') as f:
        f.write("This is a test file.")

    extractor = MetadataExtractor()

    print(f"--- Testing with a simple text file: {test_file_path} ---")
    metadata = extractor.extract(test_file_path)

    if metadata:
        for key, value in metadata.items():
            print(f"  {key}: {value}")
    else:
        print("  Failed to extract metadata.")

    # Note: Testing with a real image would require a sample image file.
    # For example:
    # print("\n--- Testing with a (non-existent) image file ---")
    # image_metadata = extractor.extract('sample.jpg')
    # if image_metadata:
    #     print("  Extracted image metadata.")
    # else:
    #     print("  Could not extract metadata (as expected, file doesn't exist).")

    # Clean up the dummy file and directory
    os.remove(test_file_path)
    os.rmdir(test_dir)
    print(f"\n--- Test complete. Cleaned up {test_file_path} and {test_dir} ---")
