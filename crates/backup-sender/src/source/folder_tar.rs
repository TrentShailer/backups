use core::num::TryFromIntError;
use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    process::Command,
};

use serde::{Deserialize, Serialize};
use shared::{Cadance, Metadata, MetadataString};
use thiserror::Error;
use tracing::{error, warn};

use crate::Backup;

use super::BackupSource;

/// Tar a folder and back it up.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct FolderTar {
    /// The path to the folder.
    pub folder_path: PathBuf,

    /// The service name.
    pub service_name: MetadataString<128>,

    /// The cadances to backup this source.
    pub cadance: Vec<Cadance>,
}

impl BackupSource for FolderTar {
    type Error = BackupFolderError;

    fn get_backup(&self, cadance: Cadance) -> Result<Backup, Self::Error> {
        let folder_metadata = fs::metadata(&self.folder_path)
            .map_err(|e| BackupFolderError::Io(e, "get folder metadata"))?;
        if !folder_metadata.is_dir() {
            return Err(BackupFolderError::NotDirectory);
        }

        let path_str = match self.folder_path.to_str() {
            Some(path) => path,
            None => return Err(BackupFolderError::NotUnicode),
        };
        let archive_name = format!("{}-{:?}.tar", self.service_name.as_string(), cadance);

        let output = Command::new("tar")
            .args(["-cf", &archive_name, path_str])
            .output()
            .map_err(BackupFolderError::RunCommand)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(BackupFolderError::CommandErrored(error));
        }

        let file = OpenOptions::new()
            .read(true)
            .open(&archive_name)
            .map_err(|e| BackupFolderError::Io(e, "open archive"))?;
        let file_metadata = file
            .metadata()
            .map_err(|e| BackupFolderError::Io(e, "get archive metadata"))?;
        let file_size = file_metadata.len();

        let metadata = Metadata::new(
            file_size,
            self.service_name,
            cadance,
            MetadataString::try_from("tar").unwrap(),
        );

        let backup = Backup {
            metadata,
            reader: Box::new(file),
        };

        Ok(backup)
    }

    fn cadance(&self) -> &[Cadance] {
        &self.cadance
    }

    fn service_name(&self) -> String {
        self.service_name.as_string()
    }

    fn cleanup(&self, metadata: Metadata) {
        let service = self.service_name.as_string();
        let cadance = metadata.cadance;

        let archive_name = format!("{}-{:?}.tar", service, metadata.cadance);

        if let Err(e) = fs::remove_file(archive_name) {
            warn!("Failed to cleanup {service}::{cadance:?} : {e}")
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum BackupFolderError {
    #[error("Failed to {0}: {1}")]
    Io(#[source] io::Error, &'static str),

    #[error("Folder path was not a directory")]
    NotDirectory,

    #[error("Folder path was invalid unicode")]
    NotUnicode,

    #[error("Failed to run command:\n{0}")]
    RunCommand(#[source] io::Error),

    #[error("Command output was error:\n{0}")]
    CommandErrored(String),

    #[error("Backup was larger than u64::MAX: {0}")]
    BackupTooLarge(#[from] TryFromIntError),
}
