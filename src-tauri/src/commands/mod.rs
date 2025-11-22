// Tauri commands module
use crate::models::{Config, ProcessResult, FileInfo};
use crate::services::{
    ConfigManager, FileProcessor, RuleEngine, 
    DefaultMetadataExtractor, DefaultFileOperations, FileOperations
};
use std::path::PathBuf;
use tracing::{info, warn, error};

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
/// Requirement 8.1, 8.5: Log all file operations for troubleshooting
#[tauri::command]
pub async fn process_files(files: Vec<String>) -> Result<Vec<ProcessResult>, String> {
    info!("process_files command called with {} files", files.len());
    
    // Load configuration
    let config_manager = ConfigManager::new(get_config_path());
    let config = match config_manager.load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully with {} rules", cfg.rules.len());
            cfg
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(format!("Failed to load configuration: {}", e));
        }
    };
    
    // Create rule engine with loaded rules
    let rule_engine = RuleEngine::new(config.rules.clone());
    info!("Rule engine initialized with {} rules", config.rules.len());
    
    // Create metadata extractor and file operations
    let metadata_extractor = Box::new(DefaultMetadataExtractor);
    let file_ops = Box::new(DefaultFileOperations);
    
    // Create file processor
    let mut processor = FileProcessor::new(rule_engine, metadata_extractor, file_ops);
    
    // Set default destination if configured
    if !config.default_destination.is_empty() {
        info!("Default destination set to: {}", config.default_destination);
        processor.set_default_destination(config.default_destination.clone());
    }
    
    // Set preview mode
    processor.set_preview_mode(config.preview_mode);
    info!("Preview mode: {}", config.preview_mode);
    
    // Process the files
    let results = processor.process_files(files);
    
    // Log summary
    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    info!(
        "process_files command completed: {} total, {} success, {} failure",
        results.len(),
        success_count,
        failure_count
    );
    
    // Log failures for troubleshooting
    for result in &results {
        if !result.success {
            warn!(
                "File processing failed: {} - {}",
                result.source_path,
                result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
    }
    
    Ok(results)
}

/// Load configuration command
/// Loads the configuration from the TOML file
/// Requirement 5.1: Load config from TOML file on startup
/// Requirement 5.4: Display error message and use default config on invalid TOML
#[tauri::command]
pub async fn load_config() -> Result<Config, String> {
    info!("load_config command called");
    
    let config_manager = ConfigManager::new(get_config_path());
    let config = match config_manager.load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully with {} rules", cfg.rules.len());
            cfg
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(format!("Failed to load configuration: {}", e));
        }
    };
    
    Ok(config)
}

/// Save configuration command
/// Saves the configuration to the TOML file
/// Requirement 5.2: Save changes to TOML file
#[tauri::command]
pub async fn save_config(config: Config) -> Result<(), String> {
    info!("save_config command called with {} rules", config.rules.len());
    
    let config_manager = ConfigManager::new(get_config_path());
    match config_manager.save(&config) {
        Ok(_) => {
            info!("Configuration saved successfully");
            Ok(())
        },
        Err(e) => {
            error!("Failed to save configuration: {}", e);
            Err(format!("Failed to save configuration: {}", e))
        }
    }
}

/// Get file info command
/// Returns information about a file (name, size, modification time)
/// Requirement: 全般 - Utility command for file information
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    info!("get_file_info command called for path: {}", path);
    
    let file_ops = DefaultFileOperations;
    match file_ops.get_file_info(&path) {
        Ok(file_info) => {
            info!(
                "File info retrieved successfully: {} ({} bytes)",
                file_info.name,
                file_info.size
            );
            Ok(file_info)
        },
        Err(e) => {
            error!("Failed to get file info for {}: {}", path, e);
            Err(format!("Failed to get file info: {}", e))
        }
    }
}
