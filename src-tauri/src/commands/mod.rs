// Tauri commands module
use crate::models::{Config, ProcessResult};

#[tauri::command]
pub async fn process_files(files: Vec<String>) -> Result<Vec<ProcessResult>, String> {
    // TODO: Implement in later tasks
    Ok(vec![])
}

#[tauri::command]
pub async fn load_config() -> Result<Config, String> {
    // TODO: Implement in later tasks
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn save_config(config: Config) -> Result<(), String> {
    // TODO: Implement in later tasks
    Ok(())
}

#[tauri::command]
pub async fn get_file_info(path: String) -> Result<String, String> {
    // TODO: Implement in later tasks
    Ok(path)
}
