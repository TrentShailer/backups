use std::io::{self, Write};

use chrono::{DateTime, Local};
use thiserror::Error;

pub async fn encrypt_file(
    file: &[u8],
    cert: &age::x25519::Recipient,
) -> Result<Vec<u8>, EncryptError> {
    let encryptor = match age::Encryptor::with_recipients(vec![Box::new(cert.clone())]) {
        Some(v) => v,
        None => return Err(EncryptError::NoRecipiantError),
    };

    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_async_output(&mut encrypted).await?;
    writer.write_all(file)?;
    writer.finish()?;

    Ok(encrypted)
}

pub fn get_file_name() -> String {
    let datetime: DateTime<Local> = Local::now();
    datetime.format("%Y-%m-%d_%H-%M-%S.backup").to_string()
}

#[derive(Debug, Error)]
pub enum EncryptError {
    #[error("AgeWrapError[br]{0}")]
    AgeWrapError(#[from] age::EncryptError),
    #[error("WriteError[br]{0}")]
    WriteError(#[from] io::Error),
    #[error("NoRecipiantError")]
    NoRecipiantError,
}
