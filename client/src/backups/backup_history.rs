pub mod history;
pub mod service_backup_history;

use crate::config::ProgramConfig;

use self::history::{History, LoadHistoryError, WriteHistoryError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct ChannelData {
    pub service_name: String,
    pub backup_name: String,
    pub time_backed_up: std::time::SystemTime,
}

pub fn load_backup_history(config: &ProgramConfig) -> Result<History, BackupHistoryError> {
    let mut backup_history: History = History::load()?;

    backup_history.add_missing_entries(config);

    backup_history.save()?;

    Ok(backup_history)
}

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

#[derive(Error, Debug)]
pub enum BackupHistoryError {
    #[error("LoadHistoryError[br]{0}")]
    LoadHistoryError(#[from] LoadHistoryError),
    #[error("WriteHistoryError[br]{0}")]
    WriteHistoryError(#[from] WriteHistoryError),
}
