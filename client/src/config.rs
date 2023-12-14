pub mod certificate;
pub mod program_config;
pub mod raw_config;
pub mod tls_config;

pub use program_config::ProgramConfig;
pub use tls_config::TlsConfig;

use std::{fs, io};
use thiserror::Error;

use crate::config::raw_config::RawConfig;

use self::{program_config::ParseProgramConfigError, tls_config::ParseTlsConfigError};

const CONFIG_PATH: &str = "./config.toml";

pub struct Config {
    pub program_config: ProgramConfig,
    pub tls_config: TlsConfig,
}

impl Config {
    pub fn load() -> Result<Self, ConfigLoadError> {
        let contents = fs::read_to_string(CONFIG_PATH)
            .map_err(|error| ConfigLoadError::FileReadError(error))?;

        let raw_config: RawConfig = toml::from_str(contents.as_str())
            .map_err(|error| ConfigLoadError::DeserialzeError(error))?;

        let program_config = ProgramConfig::try_from(&raw_config)?;

        let tls_config = TlsConfig::try_from(&raw_config)?;

        Ok(Self {
            program_config,
            tls_config,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("FileReadError\n{0}")]
    FileReadError(#[source] io::Error),
    #[error("DeserialzeError\n{0}")]
    DeserialzeError(#[source] toml::de::Error),
    #[error("ParseProgramConfigError\n{0}")]
    ParseProgramConfigError(#[from] ParseProgramConfigError),
    #[error("ParseTlsConfigError\n{0}")]
    ParseTlsConfigError(#[from] ParseTlsConfigError),
}
