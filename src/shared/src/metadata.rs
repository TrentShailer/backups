use serde::{Deserialize, Serialize};

/// Backup Metadata containing relevant information about the payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BackupMetadata {
    /// Backup size in bytes.
    pub backup_size: usize,

    /// The file's name.
    /// May contain filename invalid characters.
    pub file_name: String,

    /// The file's associated service name.
    /// May contain filename invalid characters.
    pub service_name: String,

    /// The file's associated backup name.
    /// May contain filename invalid characters.
    pub backup_name: String,

    /// The maximum number of files that should exist for that backup type.
    pub max_files: usize,
}
