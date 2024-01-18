use std::io;

use crate::backup_config::BackupConfig;

use blake3::Hash;
use chrono::Local;
use futures_rustls::TlsConnector;
use rustls_pki_types::ServerName;
use serde::{Deserialize, Serialize};
use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::Command,
};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub file_size: usize,
    pub file_hash: Hash,
    pub file_name: String,
    pub service_name: String,
    pub backup_name: String,
}

pub async fn make_backup(
    backup_config: &BackupConfig,
    connector: &TlsConnector,
    domain: ServerName<'static>,
) -> Result<(), MakeBackupError> {
    let file = get_file(backup_config).await?;
    let file_hash = blake3::hash(&file);
    let payload = Payload {
        file_size: file.len(),
        file_hash,
        file_name: Local::now().format("%Y-%m-%d_%H-%M-%S.backup").to_string(),
        service_name: backup_config.service_name.clone(),
        backup_name: backup_config.backup_name.to_string(),
    };

    let payload_string = toml::to_string(&payload)?;

    let stream = TcpStream::connect((
        backup_config.socket_address.clone(),
        backup_config.socket_port,
    ))
    .await
    .map_err(MakeBackupError::Connect)?;

    let mut stream = connector
        .connect(domain, stream)
        .await
        .map_err(MakeBackupError::Connect)?;

    stream
        .write_all(payload_string.as_bytes())
        .await
        .map_err(MakeBackupError::Write)?;

    loop {
        stream
            .write_all(&file)
            .await
            .map_err(MakeBackupError::Write)?;

        let mut response: Vec<u8> = vec![0; 32];
        let response_size = stream
            .read(&mut response)
            .await
            .map_err(MakeBackupError::Read)?;
        if &response[0..response_size] == b"exit" {
            break;
        }
    }

    Ok(())
}

pub async fn get_file(config: &BackupConfig) -> Result<Vec<u8>, GetFileError> {
    let args = [
        "exec",
        config.container_name.as_str(),
        "pg_dump",
        "-U",
        config.postgres_username.as_str(),
        "-d",
        config.postgres_database.as_str(),
    ];
    let output = Command::new("docker")
        .args(args)
        .output()
        .await
        .map_err(GetFileError::RunCommand)?;
    if !output.status.success() {
        let err_str = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(GetFileError::CommandResult(err_str));
    }

    Ok(output.stdout)
}

#[derive(Debug, Error)]
pub enum GetFileError {
    #[error("RunCommandError:\n{0}")]
    RunCommand(#[source] io::Error),
    #[error("CommandResultError:\n{0}")]
    CommandResult(String),
}

#[derive(Debug, Error)]
pub enum MakeBackupError {
    #[error("WriteError:\n{0}")]
    Write(#[source] io::Error),
    #[error("ReadError:\n{0}")]
    Read(#[source] io::Error),
    #[error("ConnectionError:\n{0}")]
    Connect(#[source] io::Error),
    #[error("SerializePayloadError:\n{0}")]
    SerializePayload(#[from] toml::ser::Error),
    #[error("GetfileError: {0}")]
    GetFile(#[from] GetFileError),
}
