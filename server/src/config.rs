mod backup_config;
mod certificate;
mod program_config;
mod raw_config;
mod tls_config;

use std::{fs, io};

pub use program_config::{ProgramConfig, ServiceConfig};
pub use raw_config::RawConfig;
use thiserror::Error;
pub use tls_config::TlsConfig;

use self::tls_config::ParseTlsConfigError;

const CONFIG_PATH: &str = "./config.toml";

pub fn load_config() -> Result<(TlsConfig, ProgramConfig), ConfigLoadError> {
    let contents = fs::read_to_string(CONFIG_PATH)?;

    let raw_config: RawConfig = toml::from_str(contents.as_str())?;

    let program_config = ProgramConfig::from(&raw_config);
    let tls_config = TlsConfig::try_from(&raw_config)?;

    Ok((tls_config, program_config))
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("FileReadError[br]{0}")]
    FileReadError(#[from] io::Error),
    #[error("DeserialzeError[br]{0}")]
    DeserialzeError(#[from] toml::de::Error),
    #[error("ParseTlsConfigError[br]{0}")]
    ParseTlsConfigError(#[from] ParseTlsConfigError),
}
