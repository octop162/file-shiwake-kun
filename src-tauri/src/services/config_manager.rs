use crate::models::Config;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn load(&self) -> Result<Config, String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }

    pub fn save(&self, _config: &Config) -> Result<(), String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }

    pub fn validate(&self, _config: &Config) -> Result<(), String> {
        // TODO: Implement in later tasks
        Ok(())
    }
}
