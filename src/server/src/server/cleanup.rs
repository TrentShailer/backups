use std::{fs, io, path::PathBuf, time::SystemTime};

use shared::BackupMetadata;
use thiserror::Error;

use crate::BACKUP_PATH;

use super::Server;

impl Server {
    /// Cleans up that backup directory by removing files over the limit.
    pub(super) fn cleanup(metadata: &BackupMetadata) -> Result<(), Error> {
        // Enforce 64-bit usize to make conversions between u64 and usize safe
        if usize::BITS != 64 {
            panic!("usize is not 64-bits");
        }

        let max_files = metadata.max_files as usize;

        let path = PathBuf::from(BACKUP_PATH)
            .join(&metadata.service_name)
            .join(&metadata.backup_name);

        // Get metadata from all the files in the directory
        let mut files: Vec<(SystemTime, PathBuf)> = fs::read_dir(path)
            .map_err(Error::ReadDir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let metadata = entry.metadata().ok()?;

                if !metadata.is_file() {
                    return None;
                }

                let created = metadata.created().ok()?;

                Some((created, entry.path()))
            })
            .collect();

        // Sort by age
        files.sort_by(|a, b| a.0.cmp(&b.0));

        // If we are under the limit, we are fine
        if files.len() <= max_files {
            return Ok(());
        }

        // The number of files over the limit
        let file_overflow = files.len() - max_files;

        // Remove the files
        let files_to_delete = &files[..file_overflow];
        for (_, file) in files_to_delete {
            fs::remove_file(file).map_err(Error::RemoveFile)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read directory:\n{0}")]
    ReadDir(#[source] io::Error),

    #[error("Failed to remove file:\n{0}")]
    RemoveFile(#[source] io::Error),
}
