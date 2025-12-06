import json
import os
import logging
from typing import Dict, Any

logger = logging.getLogger(__name__)

class ConfigManager:
    """
    Manages loading and saving of the application configuration.
    Handles JSON file operations.
    """
    def __init__(self, config_path: str = 'config.json'):
        """
        Initializes the ConfigManager.

        Args:
            config_path (str): The path to the configuration file.
        """
        self.config_path = config_path
        logger.debug(f"ConfigManager initialized with path: {self.config_path}")

    def get_default_config(self) -> Dict[str, Any]:
        """
        Returns the default configuration structure.

        Returns:
            A dictionary representing the default configuration.
        """
        return {
            "default_destination": os.path.join(os.path.expanduser("~"), "Unsorted"),
            "preview_mode": True,
            "log_path": "file-shiwake-kun.log",
            "rules": [],
            "last_selected_rule_id": None
        }

    def load_config(self) -> Dict[str, Any]:
        """
        Loads the configuration from the JSON file.
        If the file doesn't exist, it creates a default configuration.

        Returns:
            A dictionary containing the application configuration.
        """
        if not os.path.exists(self.config_path):
            logger.info(f"Config file not found at {self.config_path}. Creating default.")
            return self.create_default_config()
        
        try:
            logger.info(f"Loading config from {self.config_path}")
            with open(self.config_path, 'r', encoding='utf-8') as f:
                config = json.load(f)
            # Ensure all default keys exist
            default_config = self.get_default_config()
            for key, value in default_config.items():
                if key not in config:
                    config[key] = value
            return config
        except (json.JSONDecodeError, IOError) as e:
            logger.error(f"Error loading {self.config_path}: {e}. Creating a new default config.")
            return self.create_default_config()

    def save_config(self, config: Dict[str, Any]) -> None:
        """
        Saves the given configuration to the JSON file.

        Args:
            config: A dictionary containing the application configuration.
        """
        try:
            logger.debug(f"Saving config to {self.config_path}")
            with open(self.config_path, 'w', encoding='utf-8') as f:
                json.dump(config, f, ensure_ascii=False, indent=2)
        except IOError as e:
            logger.error(f"Error saving config to {self.config_path}: {e}")

    def create_default_config(self) -> Dict[str, Any]:
        """
        Creates and saves a default configuration file.

        Returns:
            A dictionary representing the default configuration.
        """
        logger.info(f"Creating default configuration file at {self.config_path}")
        default_config = self.get_default_config()
        self.save_config(default_config)
        return default_config

if __name__ == '__main__':
    # Example usage and testing
    cm = ConfigManager('test_config.json')
    
    # Clean up previous test file if it exists
    if os.path.exists('test_config.json'):
        os.remove('test_config.json')

    # Test loading (should create a default)
    print("--- Loading config for the first time ---")
    config = cm.load_config()
    assert config['preview_mode'] is True
    assert config['rules'] == []
    print("Config loaded/created successfully:")
    print(json.dumps(config, indent=2))
    
    # Test saving
    print("\n--- Modifying and saving config ---")
    config['preview_mode'] = False
    new_rule = {
        "id": "rule-001",
        "name": "Test Rule",
        "priority": 1,
        "operation": "copy",
        "destination_pattern": "/tmp/test",
        "conditions": [{"field": "extension", "operator": "==", "value": ".txt"}]
    }
    config['rules'].append(new_rule)
    cm.save_config(config)
    print("Config saved.")

    # Test loading the saved config
    print("\n--- Reloading config ---")
    reloaded_config = cm.load_config()
    assert reloaded_config['preview_mode'] is False
    assert len(reloaded_config['rules']) == 1
    assert reloaded_config['rules'][0]['id'] == "rule-001"
    print("Reloaded config matches saved config.")
    print(json.dumps(reloaded_config, indent=2))

    # Clean up the test file
    os.remove('test_config.json')
    print("\n--- Test complete. Cleaned up test_config.json ---")
