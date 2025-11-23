import datetime
from typing import List, Dict, Any, Tuple, Optional
import os # Added this import

class RuleEngine:
    """
    Processes metadata against a set of rules to find all matching rules
    and determine their destination paths.
    """

    def __init__(self, rules: List[Dict[str, Any]]):
        """
        Initializes the RuleEngine with a list of rules.

        Args:
            rules: A list of rule dictionaries.
        """
        self.rules = []
        for rule in rules:
            if 'destination_pattern' in rule and isinstance(rule['destination_pattern'], str):
                rule['destination_pattern'] = rule['destination_pattern'].replace('\\', '/')
            self.rules.append(rule)

    def find_all_matches(self, metadata: Dict[str, Any]) -> List[Tuple[Dict[str, Any], str]]:
        """
        Finds all rules that match the file's metadata.

        Args:
            metadata: A dictionary of the file's metadata.

        Returns:
            A list of tuples, where each tuple contains a matched rule and its
            calculated destination path. Returns an empty list if no rules match.
        """
        matches = []
        for rule in self.rules:
            if self._check_conditions(metadata, rule.get('conditions', [])):
                destination_path = self._format_destination(metadata, rule['destination_pattern'])
                matches.append((rule, destination_path))
        return matches

    def _check_conditions(self, metadata: Dict[str, Any], conditions: List[Dict[str, Any]]) -> bool:
        """
        Checks if all conditions for a rule are met by the metadata.
        """
        if not conditions: # If a rule has no conditions, it's a match
            return True
            
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
                return str(metadata_value).lower() == str(rule_value).lower()
            elif operator == '!=':
                return str(metadata_value).lower() != str(rule_value).lower()
            elif operator == 'in':
                # Handles "extension in ['.jpg', '.png']"
                return str(metadata_value).lower() in [str(v).lower() for v in rule_value]
            elif operator == 'not in':
                return str(metadata_value).lower() not in [str(v).lower() for v in rule_value]
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
        {"id": "rule-001", "name": "Sort All Images", "operation": "move", "destination_pattern": "D:/Images/{filename}", "conditions": [{"field": "extension", "operator": "in", "value": [".jpg", ".png"]}]},
        {"id": "rule-002", "name": "Sort JPGs by Year", "operation": "copy", "destination_pattern": "D:/JPGs/{year}/{filename}", "conditions": [{"field": "extension", "operator": "==", "value": ".jpg"}]}
    ]

    engine = RuleEngine(test_rules)
    
    # --- Test Case 1: Matching Photo (should match both rules) ---
    print("\n--- Test Case 1: Matching Photo ---")
    photo_metadata = {
        'filename': 'IMG_1234.jpg', 'extension': '.jpg',
        'capture_date': datetime.datetime(2023, 10, 27)
    }
    
    matches = engine.find_all_matches(photo_metadata)
    
    assert len(matches) == 2
    assert matches[0][0]['id'] == 'rule-001'
    assert matches[1][0]['id'] == 'rule-002'
    print(f"Found {len(matches)} matches, as expected.")
    print(f"  Match 1: {matches[0][0]['name']} -> {matches[0][1]}")
    print(f"  Match 2: {matches[1][0]['name']} -> {matches[1][1]}")


    # --- Test Case 2: Matching PNG (should match one rule) ---
    print("\n--- Test Case 2: Matching PNG ---")
    png_metadata = {
        'filename': 'logo.png', 'extension': '.png',
        'created_at': datetime.datetime(2023, 11, 1)
    }
    
    matches = engine.find_all_matches(png_metadata)
    
    assert len(matches) == 1
    assert matches[0][0]['id'] == 'rule-001'
    print(f"Found {len(matches)} match, as expected.")

    # --- Test Case 3: No Match ---
    print("\n--- Test Case 3: No Match ---")
    text_metadata = {'filename': 'notes.txt', 'extension': '.txt'}
    
    matches = engine.find_all_matches(text_metadata)
    
    assert len(matches) == 0
    print("No rule matched, as expected.")
    
    print("\n--- Rule Engine Test Complete ---")
