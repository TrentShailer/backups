use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use log::error;
use owo_colors::OwoColorize;
use thiserror::Error;
use tokio::{select, sync::RwLock, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{
    endpoint::{self, Endpoint, MakeBackup},
    history::{self, History},
    scheduler_config::BackupName,
    service::{GetFile, GetFileError, ServiceConfig},
};

pub async fn backup_task(
    endpoint: Arc<Endpoint>,
    service_config: ServiceConfig,
    name: BackupName,
    max_files: usize,
    sleep_duration: Duration,
    history: Arc<RwLock<History>>,
    cancel_token: CancellationToken,
) {
    let history_reader = history.read().await;
    let last_backed_up = history_reader.last_backed_up(&name);
    drop(history_reader);

    let time_since_last_backed_up = match SystemTime::now().duration_since(last_backed_up) {
        Ok(v) => v,
        Err(e) => e.duration(),
    };

    if time_since_last_backed_up > sleep_duration {
        loop {
            let result = make_backup(&endpoint, &service_config, &name, max_files, &history).await;

            if result.is_ok() {
                break;
            }

            let error = result.err().unwrap();
            error!(
                "[{}]\nFailed to make backup: {}",
                name.to_string().red(),
                error
            );

            select! {
                _ = cancel_token.cancelled() => break,
                _ = sleep(Duration::from_secs(60 * 10)) => {}
            }
        }
    }

    loop {
        select! {
            _ = cancel_token.cancelled() => break,
            _ = sleep(sleep_duration) => {}
        }

        loop {
            let result = make_backup(&endpoint, &service_config, &name, max_files, &history).await;

            if result.is_ok() {
                break;
            }

            let error = result.err().unwrap();
            error!(
                "[{}]\nFailed to make backup: {}",
                name.to_string().red(),
                error
            );

            select! {
                _ = cancel_token.cancelled() => break,
                _ = sleep(Duration::from_secs(60 * 10)) => {}
            }
        }
    }
}

async fn make_backup(
    endpoint: &Arc<Endpoint>,
    service_config: &ServiceConfig,
    name: &BackupName,
    max_files: usize,
    history: &Arc<RwLock<History>>,
) -> Result<(), MakeBackupError> {
    let file = Arc::new(service_config.get_file().await?);

    let file = Arc::clone(&file);

    endpoint.make_backup(name, max_files, &file).await?;

    let mut guard = history.write().await;
    guard.update(name).await?;
    drop(guard);

    Ok(())
}

#[derive(Debug, Error)]
pub enum MakeBackupError {
    #[error("GetFileError: {0}")]
    GetFile(#[from] GetFileError),
    #[error("MakeBackupError: {0}")]
    MakeBackup(#[from] endpoint::MakeBackupError),
    #[error("UpdateHistoryError:\n{0}")]
    UpdateHistory(#[from] history::SaveError),
}
