use thiserror::Error;

pub struct Config {
    pub port: String,
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("Error loading environment variable '{0}': {1}")]
    DotEnvyError(String, #[source] dotenvy::Error),
}

fn get_env_variable(key: &str) -> Result<String, ConfigLoadError> {
    match dotenvy::var(key) {
        Ok(variable) => Ok(variable),
        Err(error) => Err(ConfigLoadError::DotEnvyError(String::from(key), error)),
    }
}

pub fn load_config() -> Result<Config, ConfigLoadError> {
    let port = get_env_variable("PORT")?;

    Ok(Config { port })
}
