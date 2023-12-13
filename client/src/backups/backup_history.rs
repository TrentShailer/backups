pub mod backup_history;
pub mod history;
pub mod service_backup_history;

use self::history::{History, LoadHistoryError, WriteError};
use crate::config::config_types::Config;
use thiserror::Error;
use tracing::{span, trace};

pub struct ChannelData {
    pub service_name: String,
    pub backup_name: String,
    pub time_backed_up: std::time::SystemTime,
}

#[derive(Error, Debug)]
pub enum BackupHistoryError {
    #[error("Failed to load history file: {0}")]
    LoadError(#[from] LoadHistoryError),
    #[error("Failed to write history file: {0}")]
    WriteError(#[from] WriteError),
}

pub fn load_backup_history(config: &Config) -> Result<History, BackupHistoryError> {
    let span = span!(tracing::Level::TRACE, "load_backup_history");
    let _ = span.enter();

    trace!("Started loading");

    let mut backup_history: History = History::load()?;
    trace!("Loaded history");

    backup_history.add_missing_entries(config);
    trace!("Added missing entries");

    backup_history.save()?;
    trace!("Saved history");

    Ok(backup_history)
}
