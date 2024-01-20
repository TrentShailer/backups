use std::path::PathBuf;

use futures_rustls::server::TlsStream;
use log::{error, warn};
use smol::{
    fs::{self},
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use thiserror::Error;

use crate::{
    cleanup::{self, cleanup},
    payload::Payload,
    BACKUP_PATH,
};

pub async fn handler(stream: &mut TlsStream<TcpStream>) -> Result<(), HandleError> {
    let mut payload_buffer: Vec<u8> = vec![0; 1024];
    let buffer_size = stream
        .read(&mut payload_buffer)
        .await
        .map_err(HandleError::Read)?;
    let payload_string = String::from_utf8_lossy(&payload_buffer[0..buffer_size]);
    let payload: Payload = toml::from_str(&payload_string).map_err(HandleError::Deserialize)?;

    stream
        .write_all("ready".as_bytes())
        .await
        .map_err(HandleError::Write)?;

    let mut file: Vec<u8> = vec![0; payload.file_size];
    stream.read(&mut file).await.map_err(HandleError::Read)?;

    let backup_path = PathBuf::from(BACKUP_PATH)
        .join(&payload.service_name)
        .join(&payload.backup_name);

    fs::create_dir_all(&backup_path)
        .await
        .map_err(HandleError::CreateFolder)?;

    let backup_path = backup_path.join(payload.file_name);

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

    if let Err(e) = cleanup(
        &payload.service_name,
        &payload.backup_name,
        payload.max_files,
    )
    .await
    {
        match e {
            cleanup::Error::Io(_) => return Err(HandleError::Cleanup(e)),
            cleanup::Error::CreationTime(_) => {
                warn!("Unable to cleanup, filesystem doesn't support creation time.")
            }
        };
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum HandleError {
    #[error("ReadError:\n{0}")]
    Read(#[source] io::Error),
    #[error("WriteError:\n{0}")]
    Write(#[source] io::Error),
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
    #[error("CleanupError: {0}")]
    Cleanup(#[from] cleanup::Error),
}
