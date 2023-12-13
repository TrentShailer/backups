use crate::{backups::backup_types::BackupTypes, config::config_types::Config};

use super::{service_backup_history::ServiceBackupHistory, ChannelData};

use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::Path,
    time::{Duration, SystemTime},
};
use thiserror::Error;
use tracing::{instrument, span, trace};

const BACKUP_HISTORY_PATH: &str = "./backup_history.toml";

#[derive(Deserialize, Serialize, Clone)]
pub struct History {
    pub services: Vec<ServiceBackupHistory>,
}

impl History {
    pub fn new() -> Self {
        History { services: vec![] }
    }

    #[instrument(skip_all)]
    pub fn load() -> Result<Self, LoadHistoryError> {
        let span = span!(tracing::Level::TRACE, "History::load");
        let _ = span.enter();

        trace!("Loading history");

        let file_path = Path::new(BACKUP_HISTORY_PATH);
        if file_path.exists() {
            trace!("File exists");

            let contents =
                fs::read_to_string(file_path).map_err(|e| LoadHistoryError::ReadError(e))?;
            trace!("Read contents");

            let parsed = toml::from_str(&contents).map_err(|e| LoadHistoryError::ParseError(e))?;
            trace!("Pared contents");

            trace!("Returned parsed contents");
            return Ok(parsed);
        }

        trace!("Pared contents");
        Ok(Self::new())
    }

    pub fn find(
        &self,
        folder_name: &String,
        sub_folder_name: &String,
    ) -> Option<std::time::SystemTime> {
        let service = match self
            .services
            .iter()
            .find(|service| &service.folder_name == folder_name)
        {
            Some(v) => v,
            None => return None,
        };

        let backup = match service
            .backups
            .iter()
            .find(|backup| &backup.folder_name == sub_folder_name)
        {
            Some(v) => v,
            None => return None,
        };

        Some(backup.last_backed_up)
    }

    pub fn add_missing_entries(&mut self, config: &Config) {
        for service in config.service_config.iter() {
            let (service_folder_name, backup_folder_names) = match service {
                BackupTypes::DockerPostgres { config } => config.get_names(),
            };

            match self
                .services
                .iter_mut()
                .find(|service| service.folder_name == service_folder_name)
            {
                Some(service_history) => {
                    service_history.add_missing(backup_folder_names);
                }
                None => {
                    self.services.push(ServiceBackupHistory::new(
                        service_folder_name,
                        backup_folder_names,
                    ));
                }
            };
        }
    }

    pub fn update_history(&mut self, data: ChannelData) -> Result<(), UpdateHistoryError> {
        let service = match self
            .services
            .iter_mut()
            .find(|service| service.folder_name == data.service_name)
        {
            Some(v) => v,
            None => return Err(UpdateHistoryError::MissingService(data.service_name)),
        };

        let backup = match service
            .backups
            .iter_mut()
            .find(|backup| backup.folder_name == data.backup_name)
        {
            Some(v) => v,
            None => return Err(UpdateHistoryError::MissingBackup(data.backup_name)),
        };

        backup.last_backed_up = data.time_backed_up;

        Ok(())
    }

    pub fn save(&self) -> Result<(), WriteError> {
        let contents = toml::to_string(self)?;
        fs::write(Path::new(BACKUP_HISTORY_PATH), contents)?;

        Ok(())
    }

    pub async fn save_async(&self) -> Result<(), WriteError> {
        let contents = toml::to_string(self)?;
        tokio::fs::write(Path::new(BACKUP_HISTORY_PATH), contents).await?;

        Ok(())
    }

    pub async fn should_make_backup(
        &self,
        folder: &String,
        sub_folder: &String,
        backup_interval: Duration,
    ) -> bool {
        let last_backed_up = match self.find(folder, sub_folder) {
            Some(v) => v,
            None => {
                error!(
                    "Failed to find history for backup '{}/{}'",
                    folder, sub_folder
                );
                panic!("Failed to find history for backup");
            }
        };

        let duration_since = match SystemTime::now().duration_since(last_backed_up) {
            Ok(v) => v,
            Err(error) => {
                // last backed up was ahead by more than 5 seconds, panic
                if error.duration() > Duration::from_secs(5) {
                    error!(
                        "Clock has gone backwards by {} seconds",
                        error.duration().as_secs()
                    );
                    panic!("Clock has gone backwards");
                }

                Duration::from_secs(0)
            }
        };

        if duration_since > backup_interval {
            return true;
        }
        return false;
    }
}

// Error Types

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Failed to serialze self: {0}")]
    SerialzeError(#[from] toml::ser::Error),
    #[error("Failed to write file: {0}")]
    IOError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum LoadHistoryError {
    #[error("Failed read history file: {0}")]
    ReadError(#[source] io::Error),
    #[error("Failed to parse history file: {0}")]
    ParseError(#[source] toml::de::Error),
}

#[derive(Error, Debug)]
pub enum UpdateHistoryError {
    #[error("Failed to update history, couldn't find matching service: {0}")]
    MissingService(String),
    #[error("Failed to update history, couldn't find matching backup: {0}")]
    MissingBackup(String),
}
