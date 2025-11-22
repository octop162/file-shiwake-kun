import datetime
from typing import List, Dict, Any, Tuple, Optional
import os # Added this import

class RuleEngine:
    """
    Processes metadata against a set of rules to find a matching rule
    and determine the destination path.
    """

    def __init__(self, rules: List[Dict[str, Any]]):
        """
        Initializes the RuleEngine with a list of rules.

        Args:
            rules: A list of rule dictionaries, sorted by priority.
        """
        # Sort rules by priority, ascending. Lower number = higher priority.
        self.rules = sorted(rules, key=lambda r: r.get('priority', 999))

    def process_file(self, metadata: Dict[str, Any]) -> Tuple[Optional[Dict[str, Any]], Optional[str]]:
        """
        Finds the first rule that matches the file's metadata.

        Args:
            metadata: A dictionary of the file's metadata.

        Returns:
            A tuple containing the matched rule and the calculated destination path.
            Returns (None, None) if no rule matches.
        """
        for rule in self.rules:
            if self._check_conditions(metadata, rule.get('conditions', [])):
                destination_path = self._format_destination(metadata, rule['destination_pattern'])
                return rule, destination_path
        return None, None

    def _check_conditions(self, metadata: Dict[str, Any], conditions: List[Dict[str, Any]]) -> bool:
        """
        Checks if all conditions for a rule are met by the metadata.
        """
        for condition in conditions:
            field = condition['field']
            operator = condition['operator']
            value = condition['value']
            
            metadata_value = metadata.get(field)

            if not self._evaluate_condition(metadata_value, operator, value):
                return False  # If any condition fails, the rule doesn't match
        return True # All conditions passed

    def _evaluate_condition(self, metadata_value: Any, operator: str, rule_value: Any) -> bool:
        """
        Evaluates a single condition.
        """
        if operator == 'exists':
            return metadata_value is not None
        
        if metadata_value is None:
            return False # Most operators can't work on a non-existent value

        try:
            if operator == '==':
                return str(metadata_value) == str(rule_value)
            elif operator == '!=':
                return str(metadata_value) != str(rule_value)
            elif operator == 'in':
                # Handles "extension in ['.jpg', '.png']"
                return metadata_value in rule_value
            elif operator == 'not in':
                return metadata_value not in rule_value
            elif operator == '>':
                return metadata_value > rule_value
            elif operator == '<':
                return metadata_value < rule_value
            else:
                return False
        except (TypeError, ValueError) as e:
            # e.g., comparing different types
            print(f"Could not evaluate condition: {metadata_value} {operator} {rule_value}. Error: {e}")
            return False
            
    def _format_destination(self, metadata: Dict[str, Any], pattern: str) -> str:
        """
        Formats the destination path string using template variables from metadata.
        """
        capture_date = metadata.get('capture_date') or metadata.get('created_at')
        
        replacements = {
            '{year}': f"{capture_date.year:04d}" if capture_date else "YYYY",
            '{month}': f"{capture_date.month:02d}" if capture_date else "MM",
            '{day}': f"{capture_date.day:02d}" if capture_date else "DD",
            '{hour}': f"{capture_date.hour:02d}" if capture_date else "HH",
            '{minute}': f"{capture_date.minute:02d}" if capture_date else "mm",
            '{second}': f"{capture_date.second:02d}" if capture_date else "ss",
            '{extension}': metadata.get('extension', '').lstrip('.'),
            '{camera}': str(metadata.get('camera_model', 'UnknownCamera')),
            '{filename}': os.path.splitext(metadata.get('filename', 'UnknownFile'))[0],
        }

        # Sanitize values for path
        for key, val in replacements.items():
            # Basic sanitization to remove characters invalid in folder names
            replacements[key] = "".join(c for c in val if c.isalnum() or c in ('-', '_', ' ')).strip()

        for placeholder, value in replacements.items():
            pattern = pattern.replace(placeholder, value)
            
        return pattern

if __name__ == '__main__':
    # Example usage and testing
    print("--- Initializing Rule Engine Test ---")
    
    test_rules = [
        {
            "id": "rule-001",
            "name": "Sort Photos by Year/Month",
            "priority": 1,
            "operation": "move",
            "destination_pattern": "D:/Photos/{year}/{month}",
            "conditions": [
                {"field": "extension", "operator": "in", "value": [".jpg", ".jpeg"]},
                {"field": "capture_date", "operator": "exists", "value": None}
            ]
        },
        {
            "id": "rule-002",
            "name": "Sort Documents",
            "priority": 2,
            "operation": "copy",
            "destination_pattern": "C:/Documents/PDFs",
            "conditions": [
                {"field": "extension", "operator": "==", "value": ".pdf"}
            ]
        }
    ]

    engine = RuleEngine(test_rules)
    
    # --- Test Case 1: Matching Photo ---
    print("\n--- Test Case 1: Matching Photo ---")
    photo_metadata = {
        'filename': 'IMG_1234.jpg',
        'extension': '.jpg',
        'size': 5000000,
        'capture_date': datetime.datetime(2023, 10, 27, 14, 30, 0),
        'camera_model': 'iPhone 15 Pro'
    }
    
    rule, dest = engine.process_file(photo_metadata)
    
    assert rule is not None
    assert rule['id'] == 'rule-001'
    # On Windows, os.path.normpath would be better, but for a simple string test this is fine.
    assert dest.replace("\\", "/") == "D:/Photos/2023/10"
    print(f"Matched Rule: {rule['name']}")
    print(f"Destination: {dest}")


    # --- Test Case 2: Matching PDF ---
    print("\n--- Test Case 2: Matching PDF ---")
    pdf_metadata = {
        'filename': 'report.pdf',
        'extension': '.pdf',
        'size': 150000,
        'created_at': datetime.datetime(2023, 11, 1),
        'capture_date': None,
        'camera_model': None
    }
    
    rule, dest = engine.process_file(pdf_metadata)
    
    assert rule is not None
    assert rule['id'] == 'rule-002'
    assert dest.replace("\\", "/") == "C:/Documents/PDFs"
    print(f"Matched Rule: {rule['name']}")
    print(f"Destination: {dest}")

    # --- Test Case 3: No Match ---
    print("\n--- Test Case 3: No Match ---")
    text_metadata = {
        'filename': 'notes.txt',
        'extension': '.txt',
        'size': 1024,
        'created_at': datetime.datetime(2023, 11, 20),
        'capture_date': None,
        'camera_model': None
    }
    
    rule, dest = engine.process_file(text_metadata)
    
    assert rule is None
    assert dest is None
    print("No rule matched, as expected.")
    
    print("\n--- Rule Engine Test Complete ---")
