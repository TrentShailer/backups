use std::{io, path::Path};

use blake3::Hash;
use thiserror::Error;

pub async fn is_saved_file_valid(
    original_hash: &Hash,
    file_name: &String,
    folder: &String,
    sub_folder: &String,
    backup_path: &Path,
) -> Result<bool, SaveFileValidError> {
    let path = backup_path.join(folder).join(sub_folder).join(file_name);
    // read file
    let file = tokio::fs::read(path)
        .await
        .map_err(SaveFileValidError::ReadFileError)?;

    // hash file
    let hash = blake3::hash(&file);

    // compare hashes
    if &hash != original_hash {
        return Ok(false);
    }

    Ok(true)
}

#[derive(Error, Debug)]
pub enum SaveFileValidError {
    #[error("ReadFileError[br]{0}")]
    ReadFileError(#[source] io::Error),
}
