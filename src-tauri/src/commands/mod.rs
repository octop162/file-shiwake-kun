// Tauri commands module
use crate::models::{Config, ProcessResult, FileInfo};
use crate::services::{
    ConfigManager, FileProcessor, RuleEngine, 
    DefaultMetadataExtractor, DefaultFileOperations, FileOperations
};
use std::path::PathBuf;
use tracing::info;

#[cfg(test)]
mod tests;

/// Get the default config path for the application
fn get_config_path() -> PathBuf {
    // Use the app's config directory
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("file-shiwake-kun");
    
    config_dir.join("config.toml")
}

/// Process files command
/// Takes a list of file paths and processes them according to the configured rules
/// Requirement: 全般 - Main file processing command
#[tauri::command]
pub async fn process_files(files: Vec<String>) -> Result<Vec<ProcessResult>, String> {
    info!("process_files command called with {} files", files.len());
    
    // Load configuration
    let config_manager = ConfigManager::new(get_config_path());
    let config = config_manager.load()?;
    
    // Create rule engine with loaded rules
    let rule_engine = RuleEngine::new(config.rules.clone());
    
    // Create metadata extractor and file operations
    let metadata_extractor = Box::new(DefaultMetadataExtractor);
    let file_ops = Box::new(DefaultFileOperations);
    
    // Create file processor
    let mut processor = FileProcessor::new(rule_engine, metadata_extractor, file_ops);
    
    // Set default destination if configured
    if !config.default_destination.is_empty() {
        processor.set_default_destination(config.default_destination.clone());
    }
    
    // Set preview mode
    processor.set_preview_mode(config.preview_mode);
    
    // Process the files
    let results = processor.process_files(files);
    
    info!("process_files command completed with {} results", results.len());
    Ok(results)
}

/// Load configuration command
/// Loads the configuration from the TOML file
/// Requirement 5.1: Load config from TOML file on startup
#[tauri::command]
pub async fn load_config() -> Result<Config, String> {
    info!("load_config command called");
    
    let config_manager = ConfigManager::new(get_config_path());
    let config = config_manager.load()?;
    
    info!("Configuration loaded successfully");
    Ok(config)
}

/// Save configuration command
/// Saves the configuration to the TOML file
/// Requirement 5.2: Save changes to TOML file
#[tauri::command]
pub async fn save_config(config: Config) -> Result<(), String> {
    info!("save_config command called");
    
    let config_manager = ConfigManager::new(get_config_path());
    config_manager.save(&config)?;
    
    info!("Configuration saved successfully");
    Ok(())
}

/// Get file info command
/// Returns information about a file (name, size, modification time)
/// Requirement: 全般 - Utility command for file information
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    info!("get_file_info command called for path: {}", path);
    
    let file_ops = DefaultFileOperations;
    let file_info = file_ops.get_file_info(&path)?;
    
    info!("File info retrieved successfully for: {}", path);
    Ok(file_info)
}
