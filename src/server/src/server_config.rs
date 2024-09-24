use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

const CONFIG_PATH: &str = "./config.toml";

/// The server config.
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The address to listen for connections on.
    pub socket_address: SocketAddr,

    /// The path to the root certificate.
    pub root_ca_path: PathBuf,

    /// The path to the certificate.
    pub certificate_path: PathBuf,

    /// The path to the key.
    pub key_path: PathBuf,
}

impl ServerConfig {
    /// Tries to load the config from the file.
    pub fn try_load() -> Result<Self, Error> {
        if !Path::new(CONFIG_PATH).exists() {
            return Err(Error::NoFile);
        }

        let contents = fs::read_to_string(CONFIG_PATH).map_err(Error::Read)?;
        let config = toml::from_str(&contents)?;

        Ok(config)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Server config file does not exist.")]
    NoFile,

    #[error("Failed to read to file:\n{0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to deserialize file:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}
