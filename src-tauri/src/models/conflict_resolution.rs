use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    Overwrite,
    Skip,
    Rename,
    OverwriteAll,
    SkipAll,
    RenameAll,
}
