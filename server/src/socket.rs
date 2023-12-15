mod create_socket;
mod decrypt_file;
mod is_saved_file_valid;
mod payload;
mod save_file;
mod valid_config;

use std::io;

use log::error;
use thiserror::Error;
use tokio::io::{split, AsyncWriteExt};

use crate::{config::ProgramConfig, socket::payload::get_payload};

use self::{
    decrypt_file::{decrypt_file, DecryptError},
    is_saved_file_valid::{is_saved_file_valid, SaveFileValidError},
    payload::GetPayloadError,
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
        // read payload
        let payload = get_payload(&mut reader).await?;

        // check with own config
        if !valid_config(&payload.folder, &payload.sub_folder, &program_config) {
            return Err(ConnectionError::InvalidConfigError(
                payload.folder,
                payload.sub_folder,
            ));
        }

        // decrypt
        let file = decrypt_file(payload.file, &program_config.age_key).await?;

        // save
        save_file(
            file,
            &payload.file_name,
            &payload.folder,
            &payload.sub_folder,
            &program_config.backup_path,
        )
        .await?;

        // check saved file with hash
        if is_saved_file_valid(
            &payload.file_hash,
            &payload.file_name,
            &payload.folder,
            &payload.sub_folder,
            &program_config.backup_path,
        )
        .await?
        {
            successful = true;
            break;
        };

        // send retry
        writer
            .write_all(b"retry")
            .await
            .map_err(|e| ConnectionError::SendRetryError(e))?;

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
    #[error("DecryptError[br]{0}")]
    DecryptError(#[from] DecryptError),
    #[error("SaveFileError[br]{0}")]
    SaveFileError(#[from] SaveFileError),
    #[error("CheckSavedFileError[br]{0}")]
    CheckSavedFileError(#[from] SaveFileValidError),
    #[error("MaxmimumRetriesReached")]
    MaxmimumRetriesReached,
    #[error("SendRetryError[br]{0}")]
    SendRetryError(#[source] io::Error),
}
