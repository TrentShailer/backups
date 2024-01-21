use std::path::PathBuf;

use log::{error, warn};
use shared::TlsPayload;
use thiserror::Error;
use tokio::{
    fs,
    io::{self, AsyncReadExt},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use crate::{
    cleanup::{self, cleanup},
    BACKUP_PATH,
};

pub async fn handler(stream: &mut TlsStream<TcpStream>) -> Result<(), HandleError> {
    // payload size hint
    let mut payload_size: [u8; 8] = [0; 8];
    stream.read_exact(&mut payload_size).await.unwrap();
    let payload_size: usize = usize::from_be_bytes(payload_size);

    // payload
    let mut payload_buffer: Vec<u8> = vec![0; payload_size];
    stream.read_exact(&mut payload_buffer).await?;
    let payload_string = String::from_utf8_lossy(&payload_buffer);
    let payload: TlsPayload = toml::from_str(&payload_string)?;

    // file
    let mut file: Vec<u8> = vec![0; payload.file_size];
    stream.read_exact(&mut file).await?;

    // write file
    let backup_path = PathBuf::from(BACKUP_PATH)
        .join(&payload.service_name)
        .join(&payload.backup_name);
    fs::create_dir_all(&backup_path).await?;

    let backup_path = backup_path.join(payload.file_name);
    fs::write(&backup_path, file).await?;

    // verify file
    let contents = fs::read(&backup_path).await?;
    let hash = blake3::hash(&contents);
    if hash != payload.file_hash {
        return Err(HandleError::HashMismatch);
    }

    // cleanup
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
    #[error("IoError:\n{0}")]
    IoError(#[from] io::Error),
    #[error("DeserializeError:\n{0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("CleanupError: {0}")]
    Cleanup(#[from] cleanup::Error),
    #[error("HashMismatch")]
    HashMismatch,
}
