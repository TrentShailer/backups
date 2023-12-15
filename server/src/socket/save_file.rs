use std::{io, path::PathBuf};

use thiserror::Error;

pub async fn save_file(
    file: Vec<u8>,
    file_name: &String,
    folder: &String,
    sub_folder: &String,
    backup_path: &PathBuf,
) -> Result<(), SaveFileError> {
    // ensure path exists
    let path = backup_path.join(folder).join(sub_folder);
    tokio::fs::create_dir_all(&path)
        .await
        .map_err(|e| SaveFileError::CreatePathError(e))?;

    let path = path.join(file_name);
    tokio::fs::write(path, file)
        .await
        .map_err(|e| SaveFileError::WriteFileError(e))?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum SaveFileError {
    #[error("CreatePathError[br]{0}")]
    CreatePathError(#[source] io::Error),
    #[error("WriteFileError[br]{0}")]
    WriteFileError(#[source] io::Error),
}
