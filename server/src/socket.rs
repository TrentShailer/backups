mod create_socket;
mod is_saved_file_valid;
mod payload;
mod payload_config;
mod save_file;
mod valid_config;

use std::io;

use log::{error, warn};
use thiserror::Error;
use tokio::io::{split, AsyncWriteExt};

use crate::{config::ProgramConfig, socket::payload::get_payload};

use self::{
    is_saved_file_valid::{is_saved_file_valid, SaveFileValidError},
    payload::GetPayloadError,
    payload_config::{get_payload_config, GetPayloadConfigError},
    save_file::{save_file, SaveFileError},
    valid_config::valid_config,
};

pub use create_socket::create_socket;

const FILE_RETRIES: u8 = 5;

pub async fn handle_connection(
    stream: &mut tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    program_config: ProgramConfig,
) -> Result<(), ConnectionError> {
    let (mut reader, mut writer) = split(stream);

    let mut attempt = 0;
    let mut successful = false;
    while attempt < FILE_RETRIES && !successful {
        // read payload config
        let payload_config = get_payload_config(&mut reader).await?;

        // check with own config
        if !valid_config(
            &payload_config.folder,
            &payload_config.sub_folder,
            &program_config,
        ) {
            return Err(ConnectionError::InvalidConfigError(
                payload_config.folder,
                payload_config.sub_folder,
            ));
        }

        // send ready
        writer
            .write_all(b"ready")
            .await
            .map_err(ConnectionError::SendReadyError)?;

        // read payload
        let file = get_payload(&mut reader, &payload_config).await?;

        // save
        save_file(
            file,
            &payload_config.file_name,
            &payload_config.folder,
            &payload_config.sub_folder,
            &program_config.backup_path,
        )
        .await?;

        // check saved file with hash
        if is_saved_file_valid(
            &payload_config.file_hash,
            &payload_config.file_name,
            &payload_config.folder,
            &payload_config.sub_folder,
            &program_config.backup_path,
        )
        .await?
        {
            successful = true;
            break;
        };

        warn!(
            "[[cs]{}/{}[ce]][br]HashMismatch({})",
            payload_config.folder, payload_config.sub_folder, attempt
        );

        // send retry
        writer
            .write_all(b"retry")
            .await
            .map_err(ConnectionError::SendRetryError)?;

        attempt += 1;
    }

    if !successful {
        // Ran out of tries, return error
        return Err(ConnectionError::MaxmimumRetriesReached);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("GetPayloadError[br]{0}")]
    GetPayloadError(#[from] GetPayloadError),
    #[error("InvalidConfigError[br]{0}/{1}")]
    InvalidConfigError(String, String),
    #[error("SaveFileError[br]{0}")]
    SaveFileError(#[from] SaveFileError),
    #[error("CheckSavedFileError[br]{0}")]
    CheckSavedFileError(#[from] SaveFileValidError),
    #[error("MaxmimumRetriesReached")]
    MaxmimumRetriesReached,
    #[error("SendRetryError[br]{0}")]
    SendRetryError(#[source] io::Error),
    #[error("SendReadyError[br]{0}")]
    SendReadyError(#[source] io::Error),
    #[error("GetPayloadConfigError[br]{0}")]
    GetPayloadConfigError(#[from] GetPayloadConfigError),
}
