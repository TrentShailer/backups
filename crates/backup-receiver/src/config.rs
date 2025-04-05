use core::net::SocketAddr;
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use shared::Cadance;
use thiserror::Error;

/// The receiver's TLS config.
#[derive(Serialize, Deserialize, Default)]
pub struct TlsConfig {
    /// The path to the root certificate file.
    pub root_certificate_file: PathBuf,

    /// The path to the certificate file.
    pub certificate_file: PathBuf,

    /// The path to the private key file.
    pub private_key_file: PathBuf,
}

/// The maximum number of files for a given cadence.
#[derive(Serialize, Deserialize)]
pub struct MaximumFiles {
    pub hourly: u64,
    pub daily: u64,
    pub weekly: u64,
    pub monthly: u64,
}

impl Default for MaximumFiles {
    fn default() -> Self {
        Self {
            hourly: 24,
            daily: 7,
            weekly: 52,
            monthly: 60,
        }
    }
}

/// The receiver's limits.
#[derive(Serialize, Deserialize)]
pub struct Limits {
    /// The maximum payload size in bytes.
    pub maximum_payload_bytes: u64,

    /// The maximum number of backups that a sender is allowed to send in an hour.
    /// This is a sliding window.
    pub maximum_backups_per_hour: usize,

    /// The maximum number of files to store for each cadance.
    pub maximum_files: MaximumFiles,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            maximum_payload_bytes: 1024 * 1024 * 10, // 10 MiB,
            maximum_backups_per_hour: 64,            // 640 MiB per hour,
            maximum_files: MaximumFiles::default(),
        }
    }
}

/// The receiver's config
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The address to listen for senders on.
    pub socket_address: SocketAddr,

    /// The expected backup cadance.
    pub expected_cadance: Cadance,

    /// The receiver's TLS config.
    pub tls: TlsConfig,

    /// The receiver's limits
    pub limits: Limits,
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
            socket_address: "0.0.0.0:8080".parse().unwrap(),
            expected_cadance: Cadance::Hourly,
            tls: TlsConfig::default(),
            limits: Limits::default(),
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
