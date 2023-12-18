pub mod certificate;
pub mod program_config;
pub mod raw_config;
pub mod tls_config;

pub use program_config::ProgramConfig;
pub use tls_config::TlsConfig;

use std::{fs, io};
use thiserror::Error;

use crate::config::raw_config::RawConfig;

use self::tls_config::ParseTlsConfigError;

const CONFIG_PATH: &str = "./config.toml";

pub struct Config {
    pub program_config: ProgramConfig,
    pub tls_config: TlsConfig,
}

impl Config {
    pub fn load() -> Result<Self, ConfigLoadError> {
        let contents = fs::read_to_string(CONFIG_PATH).map_err(ConfigLoadError::FileReadError)?;

        let raw_config: RawConfig = toml::from_str(contents.as_str())?;

        let program_config = ProgramConfig::from(&raw_config);

        let tls_config = TlsConfig::try_from(&raw_config)?;

        Ok(Self {
            program_config,
            tls_config,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("FileReadError[br]{0}")]
    FileReadError(#[source] io::Error),
    #[error("DeserialzeError[br]{0}")]
    DeserialzeError(#[from] toml::de::Error),
    #[error("ParseTlsConfigError[br]{0}")]
    ParseTlsConfigError(#[from] ParseTlsConfigError),
}
