use crate::config_types::{Config, ConfigLoadError, RawConfig, TlsConfig};
use std::fs;

const CONFIG_PATH: &str = "./config.toml";

pub fn load_config() -> Result<(TlsConfig, Config), ConfigLoadError> {
    let contents =
        fs::read_to_string(CONFIG_PATH).map_err(|error| ConfigLoadError::FileReadError(error))?;

    let raw_config: RawConfig = toml::from_str(contents.as_str())
        .map_err(|error| ConfigLoadError::TomlDeserialzeError(error))?;

    let config = Config::from(&raw_config);
    let tls_config = TlsConfig::from(&raw_config);

    Ok((tls_config, config))
}
