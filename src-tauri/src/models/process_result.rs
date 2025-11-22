use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub source_path: String,
    pub destination_path: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub matched_rule: Option<String>,
}
