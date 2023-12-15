use std::{io, path::Path};

use thiserror::Error;

pub async fn save_file(
    file: Vec<u8>,
    file_name: &String,
    folder: &String,
    sub_folder: &String,
    backup_path: &Path,
) -> Result<(), SaveFileError> {
    // ensure path exists
    let path = backup_path.join(folder).join(sub_folder);
    tokio::fs::create_dir_all(&path)
        .await
        .map_err(SaveFileError::CreateDirError)?;

    let path = path.join(file_name);
    tokio::fs::write(path, file)
        .await
        .map_err(SaveFileError::WriteFileError)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum SaveFileError {
    #[error("CreateDirError[br]{0}")]
    CreateDirError(#[source] io::Error),
    #[error("WriteFileError[br]{0}")]
    WriteFileError(#[source] io::Error),
}
