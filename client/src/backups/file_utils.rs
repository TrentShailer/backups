use std::io::{self, Write};

use chrono::{DateTime, Local};
use thiserror::Error;

pub async fn encrypt_file(
    file: &Vec<u8>,
    cert: &age::x25519::Recipient,
) -> Result<Vec<u8>, EncryptError> {
    let encryptor = match age::Encryptor::with_recipients(vec![Box::new(cert.clone())]) {
        Some(v) => v,
        None => return Err(EncryptError::NoRecipiantError),
    };

    let mut encrypted = vec![];
    let mut writer = encryptor
        .wrap_async_output(&mut encrypted)
        .await
        .map_err(|e| EncryptError::AgeWrapError(e))?;
    writer
        .write_all(&file)
        .map_err(|e| EncryptError::WriteError(e))?;
    writer.finish().map_err(|e| EncryptError::WriteError(e))?;

    Ok(encrypted)
}

pub fn get_file_name() -> String {
    let datetime: DateTime<Local> = Local::now();
    datetime.format("%Y-%m-%d_%H-%M-%S").to_string()
}

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("AgeWrapError\n{0}")]
    AgeWrapError(#[source] age::EncryptError),
    #[error("WriteError\n{0}")]
    WriteError(#[source] io::Error),
    #[error("NoRecipiantError")]
    NoRecipiantError,
}
