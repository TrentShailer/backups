pub mod certificate;
pub mod program_config;
pub mod raw_config;
pub mod tls_config;

pub use program_config::ProgramConfig;
pub use tls_config::TlsConfig;

use std::{fs, io};
use thiserror::Error;
use tracing::{debug, instrument};

use crate::config::raw_config::RawConfig;

use self::{program_config::ParseProgramConfigError, tls_config::ParseTlsConfigError};

const CONFIG_PATH: &str = "./config.toml";

pub struct Config {
    pub program_config: ProgramConfig,
    pub tls_config: TlsConfig,
}

impl Config {
    #[instrument(level = "trace", skip_all, err)]
    pub fn load() -> Result<Self, ConfigLoadError> {
        debug!("Started loading config");

        let contents = fs::read_to_string(CONFIG_PATH)
            .map_err(|error| ConfigLoadError::FileReadError(error))?;
        debug!("Read config file");

        let raw_config: RawConfig = toml::from_str(contents.as_str())
            .map_err(|error| ConfigLoadError::DeserialzeError(error))?;
        debug!("Deserialzed raw config");

        let program_config = ProgramConfig::try_from(&raw_config)?;
        debug!("Parsed program config");

        let tls_config = TlsConfig::try_from(&raw_config)?;
        debug!("Parsed tls config");

        Ok(Self {
            program_config,
            tls_config,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("FileReadError -> {0}")]
    FileReadError(#[source] io::Error),
    #[error("DeserialzeError -> {0}")]
    DeserialzeError(#[source] toml::de::Error),
    #[error("ParseProgramConfigError -> {0}")]
    ParseProgramConfigError(#[from] ParseProgramConfigError),
    #[error("ParseTlsConfigError -> {0}")]
    ParseTlsConfigError(#[from] ParseTlsConfigError),
}
