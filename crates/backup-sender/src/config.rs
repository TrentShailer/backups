//! Backup Sender config
//!

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    endpoint::Endpoint,
    source::{DockerPostgres, Source},
};

/// The receiver's config
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The endpoint to send backups to.
    pub endpoint: Endpoint,

    /// The sources  to retreive backups from.
    pub sources: Vec<Source>,
}

impl Config {
    /// Tries to load a config from a toml file.
    pub fn load_toml(file_path: PathBuf) -> Result<Self, LoadConfigError> {
        if !file_path.exists() {
            return Err(LoadConfigError::NoFile);
        }

        let contents = fs::read_to_string(file_path).map_err(LoadConfigError::Read)?;
        let config = toml::from_str(&contents)?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: Endpoint::default(),
            sources: vec![Source::DockerPostgres(DockerPostgres::default())],
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("The file does not exist.")]
    NoFile,

    #[error("Failed to read the file:\n{0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to deserialize the file:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}
