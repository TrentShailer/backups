pub mod backup_history;
pub mod history;
pub mod service_backup_history;

use crate::config::ProgramConfig;

use self::history::{History, LoadHistoryError, WriteHistoryError};
use thiserror::Error;

pub struct ChannelData {
    pub service_name: String,
    pub backup_name: String,
    pub time_backed_up: std::time::SystemTime,
}

#[derive(Error, Debug)]
pub enum BackupHistoryError {
    #[error("LoadHistoryError -> {0}")]
    LoadHistoryError(#[from] LoadHistoryError),
    #[error("WriteHistoryError -> {0}")]
    WriteHistoryError(#[from] WriteHistoryError),
}

pub fn load_backup_history(config: &ProgramConfig) -> Result<History, BackupHistoryError> {
    let mut backup_history: History = History::load()?;

    backup_history.add_missing_entries(config);

    backup_history.save()?;

    Ok(backup_history)
}
