use std::io;

use blake3::Hash;
use log::debug;
use rustls::client;
use serde::Deserialize;
use thiserror::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_rustls::server::TlsStream;

use crate::config_types::Config;

#[derive(Deserialize)]
pub struct IncomingBackupConfig {
    pub folder: String,
    pub sub_folder: String,
    pub file_name: String,
    pub file_hash: Hash,
}

#[derive(Debug, Error)]
pub enum IncomingBackupConfigError {
    #[error("Failed to read client message: {0}")]
    ReadError(#[source] io::Error),
    #[error("Failed to parse backup config from client. Recieved '{0}'. Error: {1}")]
    ParseError(String, #[source] toml::de::Error),
    #[error("Couldn't finding matching folder for {0}")]
    ServiceNameError(String),
    #[error("Couldn't finding matching sub_folder for {0}")]
    BackupTypeError(String),
}

// This fn doesn't handle the error's itself, it just propegates them up to be handled further up the chain
pub async fn recieve_backup_config(
    server_config: &Config,
    stream: &mut TlsStream<TcpStream>,
) -> Result<IncomingBackupConfig, IncomingBackupConfigError> {
    let mut client_message: String = String::new();

    // read client message
    stream
        .read_to_string(&mut client_message)
        .await
        .map_err(|e| IncomingBackupConfigError::ReadError(e))?;

    debug!("{}", client_message);

    // parse message
    let backup_config: IncomingBackupConfig = toml::from_str(&client_message)
        .map_err(|e| IncomingBackupConfigError::ParseError(client_message, e))?;

    // Find matching serivce name
    let service_config = match server_config
        .service_config
        .iter()
        .find(|elem| elem.folder_name == backup_config.folder)
    {
        Some(service_config) => service_config,
        None => {
            return Err(IncomingBackupConfigError::ServiceNameError(
                backup_config.folder,
            ))
        }
    };

    // check if service config has the backup type
    if !service_config
        .backup_configs
        .iter()
        .any(|elem| elem.folder_name == backup_config.sub_folder)
    {
        return Err(IncomingBackupConfigError::BackupTypeError(
            backup_config.sub_folder,
        ));
    }

    Ok(backup_config)
}
