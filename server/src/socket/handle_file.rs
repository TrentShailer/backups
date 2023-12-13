use blake3::Hash;
use log::{info, warn};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use std::{io, iter};
use thiserror::Error;
use tokio::time::sleep;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_rustls::server::TlsStream;

use crate::config_types::Config;

use super::backup_config::IncomingBackupConfig;

#[derive(Debug, Error)]
pub enum HandleFileError {
    #[error("Failed to read encrypted file from client: {0}")]
    ReadError(#[source] io::Error),
    #[error(transparent)]
    DecryptError(#[from] DecryptFileError),
    #[error("Hahes don't match")]
    HashError,
    #[error("Failed to write file to disk: {0}")]
    WriteError(#[source] WriteFileError),
    #[error("Failed to write file, reached max retries")]
    RetriesError,
}

pub async fn handle_file(
    server_config: &Config,
    service_config: &IncomingBackupConfig,
    stream: &mut TlsStream<TcpStream>,
) -> Result<(), HandleFileError> {
    let mut encrypted: Vec<u8> = vec![];
    stream
        .read_to_end(&mut encrypted)
        .await
        .map_err(|e| HandleFileError::ReadError(e))?;

    let decrypted = decrypt_file(encrypted, &server_config.age_key).await?;

    // compare hashes
    if !hashes_match(&decrypted, &service_config.file_hash) {
        return Err(HandleFileError::HashError);
    }

    let mut write_successful = false;
    for try_count in 0..5 {
        // write to disk
        match write_file_with_checks(&decrypted, server_config, service_config).await {
            Ok(_) => {
                write_successful = true;
                if try_count != 0 {
                    info!("Attempt {} - Successfully write to file", try_count);
                }
                break;
            }
            Err(error) => match error {
                WriteFileError::HashError => {
                    warn!(
                        "Attempt {} - Failed to write file, hashes don't mach",
                        try_count
                    )
                }
                _ => return Err(HandleFileError::WriteError(error)),
            },
        }
    }

    if !write_successful {
        return Err(HandleFileError::RetriesError);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum DecryptFileError {
    #[error("Failed to create decryptor: {0}")]
    CreateDecryptorError(#[source] age::DecryptError),
    #[error("Failed to decrypt file: {0}")]
    DecryptError(#[source] age::DecryptError),
    #[error("Read the decryptor: {0}")]
    ReadDecryptorError(#[source] io::Error),
}

pub async fn decrypt_file(
    encrypted: Vec<u8>,
    key: &age::x25519::Identity,
) -> Result<Vec<u8>, DecryptFileError> {
    let decryptor = match age::Decryptor::new(&encrypted[..])
        .map_err(|e| DecryptFileError::CreateDecryptorError(e))?
    {
        age::Decryptor::Recipients(d) => d,
        _ => unreachable!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt_async(iter::once(key as &dyn age::Identity))
        .map_err(|e| DecryptFileError::DecryptError(e))?;
    reader
        .read_to_end(&mut decrypted)
        .map_err(|e| DecryptFileError::ReadDecryptorError(e))?;

    Ok(decrypted)
}

pub fn hashes_match(file: &Vec<u8>, target_hash: &Hash) -> bool {
    let hash = blake3::hash(file);
    hash.eq(target_hash)
}

#[derive(Debug, Error)]
pub enum WriteFileError {
    #[error("Failed to create path '{0}': {1}")]
    CreatePathError(String, #[source] io::Error),
    #[error("Failed to write file: {0}")]
    WriteError(#[from] WriteError),
    #[error("Failed to read written file: {0}")]
    ReadError(#[from] ReadError),
    #[error("Failed write file, hashes don't match")]
    HashError,
}

pub async fn write_file_with_checks(
    file: &Vec<u8>,
    server_config: &Config,
    service_config: &IncomingBackupConfig,
) -> Result<(), WriteFileError> {
    let folder_fragment = PathBuf::from(&service_config.folder);
    let sub_folder_fragment = PathBuf::from(&service_config.sub_folder);
    let file_name = PathBuf::from(&service_config.file_name);
    let backup_path = server_config
        .backup_path
        .join(folder_fragment)
        .join(sub_folder_fragment)
        .join(file_name);

    // ensure directories exist
    tokio::fs::create_dir_all(&backup_path)
        .await
        .map_err(|e| WriteFileError::CreatePathError(format!("{}", backup_path.display()), e))?;

    write_file(&backup_path, file).await?;

    sleep(Duration::from_millis(500)).await;

    let contents = read_file(&backup_path).await?;

    // compare hashes again
    if !hashes_match(&contents, &service_config.file_hash) {
        return Err(WriteFileError::HashError);
    }

    Ok(())
}

const SLEEP_TIME_SEC: u64 = 2;

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("Failed to read file due to unhandleable error: {0}")]
    UnhandleableReadError(#[source] io::Error),
    #[error("Failed to read file, reached max retries")]
    RetriesError,
}

async fn write_file(file_path: &PathBuf, file: &Vec<u8>) -> Result<(), WriteError> {
    let mut write_successful = false;

    // write file, with exponential backoff for some possibly recoverable errors
    for try_count in 0..10 {
        match tokio::fs::write(file_path, file).await {
            Ok(_) => {
                if try_count != 0 {
                    info!("Attempt {} - Write file successfully", try_count);
                }
                write_successful = true;
                break;
            }
            Err(error) => match error.kind() {
                io::ErrorKind::AddrInUse
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::Interrupted
                | io::ErrorKind::OutOfMemory
                | io::ErrorKind::TimedOut => {
                    warn!("Attempt {} - Failed to write file: {}", try_count, error);
                    sleep(Duration::from_secs(u64::pow(SLEEP_TIME_SEC, try_count + 1))).await;
                }
                _ => return Err(WriteError::UnhandleableReadError(error)),
            },
        }
    }

    if !write_successful {
        return Err(WriteError::RetriesError);
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Failed to read file due to unhandleable error: {0}")]
    UnhandleableReadError(#[source] io::Error),
    #[error("Failed to read file, reached max retries")]
    RetriesError,
}

async fn read_file(file_path: &PathBuf) -> Result<Vec<u8>, ReadError> {
    let mut contents: Vec<u8> = vec![];
    let mut read_successful = false;

    for try_count in 0..10 {
        match tokio::fs::read(file_path).await {
            Ok(v) => {
                contents = v;
                read_successful = true;
                break;
            }
            Err(error) => match error.kind() {
                io::ErrorKind::AddrInUse
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::Interrupted
                | io::ErrorKind::OutOfMemory
                | io::ErrorKind::TimedOut => {
                    warn!("Attempt {} - Failed to read file: {}", try_count, error);
                    sleep(Duration::from_secs(u64::pow(SLEEP_TIME_SEC, try_count + 1))).await;
                }
                _ => return Err(ReadError::UnhandleableReadError(error)),
            },
        }
    }

    if !read_successful {
        return Err(ReadError::RetriesError);
    }

    Ok(contents)
}
