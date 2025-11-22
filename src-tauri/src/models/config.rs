use serde::{Deserialize, Serialize};
use super::Rule;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub rules: Vec<Rule>,
    pub default_destination: String,
    pub preview_mode: bool,
    pub log_path: String,
}
