use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct BackupHistory {
    pub folder_name: String,
    pub last_backed_up: std::time::SystemTime,
}

impl BackupHistory {
    pub fn new(folder_name: String) -> Self {
        Self {
            folder_name,
            last_backed_up: std::time::SystemTime::UNIX_EPOCH,
        }
    }
}
