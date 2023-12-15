use std::{
    io::{self, Read},
    iter,
};

use thiserror::Error;

pub async fn decrypt_file(
    file: Vec<u8>,
    key: &age::x25519::Identity,
) -> Result<Vec<u8>, DecryptError> {
    let decryptor =
        match age::Decryptor::new(&file[..]).map_err(|e| DecryptError::CreateDecryptorError(e))? {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

    let mut decrypted = vec![];
    let mut reader = decryptor
        .decrypt_async(iter::once(key as &dyn age::Identity))
        .map_err(|e| DecryptError::DecryptError(e))?;
    reader
        .read_to_end(&mut decrypted)
        .map_err(|e| DecryptError::ReadDecryptorError(e))?;

    Ok(decrypted)
}

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error("CreateDecryptorError[br]{0}")]
    CreateDecryptorError(#[source] age::DecryptError),
    #[error("DecryptyError[br]{0}")]
    DecryptError(#[source] age::DecryptError),
    #[error("ReadDecryptorError[br]{0}")]
    ReadDecryptorError(#[source] io::Error),
}
