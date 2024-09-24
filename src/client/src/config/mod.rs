pub mod backup;

use std::{fs, io, time::Duration};

use backup::Backup;
use serde::Deserialize;
use thiserror::Error;

use crate::{endpoint::Endpoint, service::Service};

const CONFIG_PATH: &str = "./config.toml";

/// The config defining backup service-endpoint pairs, and their backup intervals.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The backup pairs.
    pub backup_pairs: Vec<BackupPair>,
}

/// A service-endpoint pair and their intervals.
#[derive(Debug, Deserialize)]
pub struct BackupPair {
    /// The service name, should be a valid directory name.
    pub service_name: String,

    /// The service that yields the backup.
    pub service: Service,

    /// The endpoint name, should be a unique for this endpoint.
    pub endpoint_name: String,

    /// The endpoint that recieves the backup.
    pub endpoint: Endpoint,

    /// The intervals for this pair.
    pub intervals: Vec<BackupIntervals>,
}

/// A given interval for a backup pair.
#[derive(Debug, Deserialize)]
pub struct BackupIntervals {
    /// The name of this backup interval, should be a valid directory name.
    pub backup_name: String,

    /// The interval between backups in seconds.
    pub interval: u64,

    /// The maximum number of files the endpoint should retain of this backup interval.
    pub max_files: usize,
}

impl Config {
    /// Load the config from the config file.
    pub fn load() -> Result<Self, Error> {
        let config_contents = fs::read_to_string(CONFIG_PATH).map_err(Error::ReadConfig)?;
        let config = toml::from_str(&config_contents)?;

        Ok(config)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to read config file:\n{0}")]
    ReadConfig(#[source] io::Error),

    #[error("Failed to deserialize config:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}

impl<'a> BackupPair {
    pub fn as_backups(&'a self) -> Vec<Backup<'a>> {
        let backups = self
            .intervals
            .iter()
            .map(|interval| Backup {
                endpoint: &self.endpoint,
                service: &self.service,
                service_name: self.service_name.clone(),
                endpoint_name: self.endpoint_name.clone(),
                backup_name: interval.backup_name.clone(),
                interval: Duration::from_secs(interval.interval),
                max_files: interval.max_files,
            })
            .collect();

        backups
    }
}
