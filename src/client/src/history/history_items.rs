use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// A struct representing the history for a single specific backup.
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub endpoint_name: String,
    pub service_name: String,
    pub backup_name: String,
    pub last_backed_up: SystemTime,
}
