// Module declarations
pub mod models;
pub mod services;
pub mod commands;

use tracing_subscriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_files,
            commands::load_config,
            commands::save_config,
            commands::get_file_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
