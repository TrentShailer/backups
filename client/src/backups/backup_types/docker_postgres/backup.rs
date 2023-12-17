use std::{
    io::{self},
    string::FromUtf8Error,
    time::SystemTime,
};

use thiserror::Error;
use tokio::{
    process::Command,
    sync::mpsc::{error::SendError, Sender},
};

use crate::{
    backups::{
        backup_history::ChannelData,
        backup_types::BackupConfig,
        file_utils::{encrypt_file, get_file_name, EncryptError},
    },
    tls::tls_client::{self, Payload, TlsClient},
};

use super::DockerPostgresBackupConfig;

pub async fn make_backup(
    config: &DockerPostgresBackupConfig,
    backup_config: &BackupConfig,
    age_cert: &age::x25519::Recipient,
    history_writer: &Sender<ChannelData>,
    tls_client: &TlsClient,
) -> Result<(), MakeBackupError> {
    let file = get_file(config).await?;
    let file_hash = blake3::hash(&file);
    let encrypted_file = encrypt_file(&file, age_cert).await?;

    let file_name = get_file_name();

    let payload = Payload {
        file_hash,
        file_name,
        folder: config.folder_name.clone(),
        sub_folder: backup_config.folder_name.clone(),
        file_size: encrypted_file.len(),
    };

    tls_client.upload_file(payload, encrypted_file).await?;

    history_writer
        .send(ChannelData {
            service_name: config.folder_name.clone(),
            backup_name: backup_config.folder_name.clone(),
            time_backed_up: SystemTime::now(),
        })
        .await?;

    Ok(())
}

pub async fn get_file(config: &DockerPostgresBackupConfig) -> Result<Vec<u8>, GetFileError> {
    let args = [
        "exec",
        config.docker_container.as_str(),
        "pg_dump",
        "-U",
        config.postgres_user.as_str(),
        "-d",
        config.postgres_database.as_str(),
    ];
    let output = Command::new("docker").args(args).output().await?;

    if !output.status.success() {
        let output_error = String::from_utf8(output.stderr.clone())?;
        return Err(GetFileError::CommandResultError(output_error));
    }

    Ok(output.stdout)
}

#[derive(Debug, Error)]
pub enum GetFileError {
    #[error("CommandError[br]{0}")]
    CommandError(#[from] io::Error),
    #[error("ComandResultError[br]{0}")]
    CommandResultError(String),
    #[error("ConvertCommandResultError[br]{0}")]
    ConvertCommandResultError(#[from] FromUtf8Error),
}

#[derive(Debug, Error)]
pub enum MakeBackupError {
    #[error("GetFileError[br]{0}")]
    GetFileError(#[from] GetFileError),
    #[error("EncryptError[br]{0}")]
    EncryptError(#[from] EncryptError),
    #[error("UploadError[br]{0}")]
    UploadError(#[from] tls_client::UploadError),
    #[error("HistorySendError[br]{0}")]
    HistroySendError(#[from] SendError<ChannelData>),
}
