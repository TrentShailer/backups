use std::{
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use log::error;
use serde::Deserialize;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Deserialize, Clone)]
pub struct BackupConfig {
    pub folder_name: String,
    pub max_files: usize,
}

impl BackupConfig {
    pub fn spawn_cleanup_task(&self, backup_path: PathBuf, parent_folder: &String) {
        let config = self.clone();
        let backup_path = backup_path.clone();
        let parent_folder = parent_folder.clone();
        tokio::spawn(async move {
            let dir = backup_path.join(&parent_folder).join(&config.folder_name);
            loop {
                if let Err(error) = Self::cleanup_task(&config, &dir).await {
                    error!(
                        "[[cs]{}/{}[ce]][br]CleanupTaskError[br]{}",
                        parent_folder.clone(),
                        config.folder_name.clone(),
                        error
                    );
                }
                sleep(Duration::from_secs(60 * 60)).await;
            }
        });
    }

    async fn cleanup_task(config: &Self, backup_path: &Path) -> Result<(), CleanupTaskError> {
        tokio::fs::create_dir_all(backup_path)
            .await
            .map_err(CleanupTaskError::CreateDirError)?;

        // load the all file info into memory
        let mut file_infos: Vec<FileInfo> = Self::get_file_infos(backup_path).await?;

        if file_infos.len() <= config.max_files {
            return Ok(());
        }

        // sort_by sorts a,b if result is lesser
        // a < b if a is older
        // since we want oldest last, we should reverse the comparison
        file_infos.sort_by(|a, b| b.created.cmp(&a.created));

        // since we know number of files > max files, this is safe
        let n_items_to_delete = file_infos.len().abs_diff(config.max_files);

        for _ in 0..n_items_to_delete {
            // pop the oldest file
            let file_info = match file_infos.pop() {
                Some(v) => v,
                None => unreachable!(),
            };

            // delete it
            tokio::fs::remove_file(file_info.path)
                .await
                .map_err(CleanupTaskError::RemoveFileError)?;
        }

        Ok(())
    }

    async fn get_file_infos(backup_path: &Path) -> Result<Vec<FileInfo>, GetFileInfoError> {
        // find the files in the dir
        let mut dir_reader = tokio::fs::read_dir(backup_path)
            .await
            .map_err(GetFileInfoError::ReadDirError)?;

        let mut file_infos: Vec<FileInfo> = vec![];

        loop {
            let maybe_entry = dir_reader
                .next_entry()
                .await
                .map_err(GetFileInfoError::NextEntryError)?;
            let entry = match maybe_entry {
                Some(v) => v,
                None => break,
            };

            let metadata = entry
                .metadata()
                .await
                .map_err(GetFileInfoError::ReadMetadataError)?;

            if !metadata.is_file() {
                continue;
            }

            let created = metadata
                .created()
                .map_err(GetFileInfoError::GetCreatedError)?;

            file_infos.push(FileInfo {
                created,
                path: entry.path(),
            });
        }
        Ok(file_infos)
    }
}

struct FileInfo {
    pub created: SystemTime,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum GetFileInfoError {
    #[error("ReadDirError[br]{0}")]
    ReadDirError(#[source] io::Error),
    #[error("NextEntryError[br]{0}")]
    NextEntryError(#[source] io::Error),
    #[error("ReadMetadataError[br]{0}")]
    ReadMetadataError(#[source] io::Error),
    #[error("GetCreatedError[br]{0}")]
    GetCreatedError(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum CleanupTaskError {
    #[error("CreateDirError[br]{0}")]
    CreateDirError(#[source] io::Error),
    #[error("GetFileInfoError[br]{0}")]
    GetFileInfoError(#[from] GetFileInfoError),
    #[error("RemoveFileError[br]{0}")]
    RemoveFileError(#[source] io::Error),
    #[error("TestError")]
    TestError,
}
