use std::path::PathBuf;

use futures_rustls::server::TlsStream;
use log::error;
use smol::{
    fs::{self},
    io::{self, AsyncReadExt},
    net::TcpStream,
};
use thiserror::Error;

use crate::payload::Payload;

const BACKUP_PATH: &str = "./backups";

pub async fn handler(stream: &mut TlsStream<TcpStream>) -> Result<(), HandleError> {
    let mut payload_buffer: Vec<u8> = vec![0; 1024];
    let buffer_size = stream
        .read(&mut payload_buffer)
        .await
        .map_err(HandleError::Read)?;
    let payload_string = String::from_utf8_lossy(&payload_buffer[0..buffer_size]);
    let payload: Payload = toml::from_str(&payload_string).map_err(HandleError::Deserialize)?;

    let mut file: Vec<u8> = vec![0; payload.file_size];
    stream.read(&mut file).await.map_err(HandleError::Read)?;

    let backup_path = PathBuf::from(BACKUP_PATH)
        .join(payload.service_name)
        .join(payload.backup_name)
        .join(payload.file_name);

    fs::create_dir_all(&backup_path)
        .await
        .map_err(HandleError::CreateFolder)?;

    fs::write(&backup_path, file)
        .await
        .map_err(HandleError::WriteFile)?;

    let contents = fs::read(&backup_path)
        .await
        .map_err(HandleError::ReadFile)?;

    let hash = blake3::hash(&contents);

    if hash != payload.file_hash {
        return Err(HandleError::HashMismatch);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum HandleError {
    #[error("ReadError:\n{0}")]
    Read(#[source] io::Error),
    #[error("DeserializeError:\n{0}")]
    Deserialize(#[source] toml::de::Error),
    #[error("CreateFolderError:\n{0}")]
    CreateFolder(#[source] io::Error),
    #[error("WriteFileError:\n{0}")]
    WriteFile(#[source] io::Error),
    #[error("ReadFileError:\n{0}")]
    ReadFile(#[source] io::Error),
    #[error("HashMismatch")]
    HashMismatch,
}
