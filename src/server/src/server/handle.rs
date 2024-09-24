use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    net::TcpStream,
    path::PathBuf,
};

use log::info;
use rustls::{ServerConnection, Stream};
use shared::BackupMetadata;
use thiserror::Error;

use crate::BACKUP_PATH;

use super::{cleanup, Server};

impl Server {
    /// Handles an connected TLS stream.
    pub(super) fn handle_connection(
        stream: &mut Stream<'_, ServerConnection, TcpStream>,
    ) -> Result<(), Error> {
        // Read metadata size hint
        let mut metdata_size = [0u8; 8];
        stream
            .read_exact(&mut metdata_size)
            .map_err(Error::ReadMetadataHint)?;
        let metadata_size = usize::from_be_bytes(metdata_size);

        // Read metadata
        let mut metadata_buffer = vec![0; metadata_size];
        stream
            .read_exact(&mut metadata_buffer)
            .map_err(Error::ReadMetadata)?;
        let metadata_string = String::from_utf8_lossy(&metadata_buffer);
        let metadata: BackupMetadata =
            toml::from_str(&metadata_string).map_err(Error::DeserializeMetadata)?;

        info!(
            "Received metadata for {}/{}",
            metadata.service_name, metadata.backup_name
        );

        // Create and open output file
        let backup_dir = PathBuf::from(BACKUP_PATH)
            .join(&metadata.service_name)
            .join(&metadata.backup_name);

        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).map_err(Error::CreateDir)?;
        }

        let backup_path = backup_dir.join(&metadata.file_name);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(backup_path)
            .map_err(Error::OpenFile)?;

        // Setup 1 KiB buffer for reading
        let mut file_buffer = [0u8; 1024];
        let mut total_bytes_read = 0;

        // Read the payload in chunks and append the chunks to the output file.
        while total_bytes_read < metadata.backup_size {
            let bytes_read = stream
                .read(&mut file_buffer[..])
                .map_err(Error::ReadPayload)?;

            file.write_all(&file_buffer[..bytes_read])
                .map_err(Error::WriteFile)?;

            total_bytes_read += bytes_read;
        }

        // cleanup
        Self::cleanup(&metadata)?;

        info!("Handled {}/{}", metadata.service_name, metadata.backup_name);

        Ok(())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to read metadata hint:\n{0}")]
    ReadMetadataHint(#[source] io::Error),

    #[error("Failed to read metadata:\n{0}")]
    ReadMetadata(#[source] io::Error),

    #[error("Failed to deserialize metadata:\n{0}")]
    DeserializeMetadata(#[source] toml::de::Error),

    #[error("Failed to read payload chunk:\n{0}")]
    ReadPayload(#[source] io::Error),

    #[error("Failed to create directory:\n{0}")]
    CreateDir(#[source] io::Error),

    #[error("Failed to open file:\n{0}")]
    OpenFile(#[source] io::Error),

    #[error("Failed to write to file:\n{0}")]
    WriteFile(#[source] io::Error),

    #[error("Failed to cleanup file overflow:\n{0}")]
    Cleanup(#[from] cleanup::Error),
}
