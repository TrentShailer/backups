mod history_items;

use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
    time::SystemTime,
};

use history_items::HistoryItem;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::backup::Backup;

const HISTORY_PATH: &str = "./history.toml";

/// History tracking for backups.
#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub history_items: Vec<HistoryItem>,
}

impl History {
    /// Load the history from the file else create a new file.
    pub fn load_or_create() -> Result<Self, Error> {
        let path = PathBuf::from(HISTORY_PATH);

        if !path.exists() {
            let history = Self {
                history_items: vec![],
            };
            history.save()?;

            return Ok(history);
        }

        let contents = fs::read_to_string(path).map_err(Error::Read)?;
        let history = toml::from_str(&contents)?;

        Ok(history)
    }

    /// Returns when a backup was last backed up, if it has never been backed up then `SystemTime::UNIX_EPOCH` will be returned.
    pub fn last_backed_up(&self, backup: &Backup) -> SystemTime {
        let maybe_item = self.history_items.iter().find(|item| {
            item.backup_name == backup.backup_name
                && item.service_name == backup.service_name
                && item.endpoint_name == backup.endpoint_name
        });

        if let Some(item) = maybe_item {
            return item.last_backed_up;
        }

        SystemTime::UNIX_EPOCH
    }

    /// Updates the time when the backup was last backed up to now.
    pub fn update(&mut self, backup: &Backup) -> Result<(), Error> {
        let maybe_item = self.history_items.iter_mut().find(|item| {
            item.backup_name == backup.backup_name
                && item.service_name == backup.service_name
                && item.endpoint_name == backup.endpoint_name
        });

        match maybe_item {
            Some(item) => item.last_backed_up = SystemTime::now(),
            None => {
                let item = HistoryItem {
                    endpoint_name: backup.endpoint_name.clone(),
                    service_name: backup.service_name.clone(),
                    backup_name: backup.backup_name.clone(),
                    last_backed_up: SystemTime::now(),
                };

                self.history_items.push(item);
            }
        };

        self.save()?;

        Ok(())
    }

    fn save(&self) -> Result<(), Error> {
        let contents = toml::to_string(self)?;
        fs::write(PathBuf::from(HISTORY_PATH), contents).map_err(Error::Write)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to serialize history:\n{0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Failed to deserialize history:\n{0}")]
    Deserialize(#[from] toml::de::Error),

    #[error("Failed to write history:\n{0}")]
    Write(#[source] io::Error),

    #[error("Failed to read history:\n{0}")]
    Read(#[source] io::Error),
}
