use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const HISTORY_PATH: &str = "./history.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub services: Vec<ServiceHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceHistory {
    pub service_name: String,
    pub backups: Vec<BackupHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupHistory {
    pub backup_name: String,
    pub last_backed_up: SystemTime,
}

impl History {
    pub fn init() -> Result<Self, InitError> {
        let path = PathBuf::from(HISTORY_PATH);
        if path.exists() {
            let mut file = File::open(path).map_err(InitError::OpenFile)?;
            let mut contents = String::new();
            File::read_to_string(&mut file, &mut contents).map_err(InitError::ReadFile)?;

            let history: Self = toml::from_str(&contents)?;

            Ok(history)
        } else {
            Ok(Self { services: vec![] })
        }
    }

    pub fn last_backed_up(&self, service_name: &str, backup_name: &str) -> SystemTime {
        let maybe_service = self
            .services
            .iter()
            .find(|service| service.service_name == service_name);

        if let Some(service) = maybe_service {
            let maybe_backup = service
                .backups
                .iter()
                .find(|backup| backup.backup_name == backup_name);

            if let Some(backup) = maybe_backup {
                backup.last_backed_up
            } else {
                SystemTime::UNIX_EPOCH
            }
        } else {
            SystemTime::UNIX_EPOCH
        }
    }

    pub async fn update(&mut self, service_name: &str, backup_name: &str) -> Result<(), SaveError> {
        let maybe_service = self
            .services
            .iter_mut()
            .find(|service| service.service_name == service_name);

        if let Some(service) = maybe_service {
            let maybe_backup = service
                .backups
                .iter_mut()
                .find(|backup| backup.backup_name == backup_name);

            if let Some(backup) = maybe_backup {
                backup.last_backed_up = SystemTime::now();
            } else {
                service.backups.push(BackupHistory {
                    backup_name: backup_name.to_string(),
                    last_backed_up: SystemTime::now(),
                });
            }
        } else {
            self.services.push(ServiceHistory {
                service_name: service_name.to_string(),
                backups: vec![BackupHistory {
                    backup_name: backup_name.to_string(),
                    last_backed_up: SystemTime::now(),
                }],
            });
        }

        Ok(self.save().await?)
    }

    async fn save(&self) -> Result<(), SaveError> {
        let contents = toml::to_string(self)?;
        smol::fs::write(PathBuf::from(HISTORY_PATH), contents).await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("OpenFileError:\n{0}")]
    OpenFile(#[source] io::Error),
    #[error("ReadFileError:\n{0}")]
    ReadFile(#[source] io::Error),
    #[error("DeserializeError:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("WriteError:\n{0}")]
    Write(#[from] smol::io::Error),
    #[error("SerializeError:\n{0}")]
    Serialize(#[from] toml::ser::Error),
}
