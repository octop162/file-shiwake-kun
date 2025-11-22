use crate::models::FileMetadata;

pub trait MetadataExtractor: Send + Sync {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String>;
}

pub struct DefaultMetadataExtractor;

impl MetadataExtractor for DefaultMetadataExtractor {
    fn extract(&self, filepath: &str) -> Result<FileMetadata, String> {
        // TODO: Implement in later tasks
        Err("Not implemented yet".to_string())
    }
}
