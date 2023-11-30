use regex::Regex;
use thiserror::Error;

pub struct Config {
    pub address: String,
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("Error loading environment variable '{0}': {1}")]
    DotEnvyError(String, #[source] dotenvy::Error),
    #[error("Error loading environment variable '{0}': Invalid format")]
    VariableFormatError(String),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
}

fn get_env_variable(key: &str) -> Result<String, ConfigLoadError> {
    match dotenvy::var(key) {
        Ok(variable) => Ok(variable),
        Err(error) => Err(ConfigLoadError::DotEnvyError(String::from(key), error)),
    }
}

pub fn load_config() -> Result<Config, ConfigLoadError> {
    let address = get_env_variable("ADDRESS")?;
    let address_regex = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{4,5}$")?;
    if !address_regex.is_match(&address) {
        return Err(ConfigLoadError::VariableFormatError(String::from(
            "ADDRESS",
        )));
    }

    Ok(Config { address })
}
