mod backup;

use std::{f32::consts::E, time::Duration};

use crate::{
    backups::backup_history::{history::History, ChannelData},
    tls::TlsClient,
};
use age::x25519::Recipient;
use log::error;
use serde::Deserialize;
use tokio::{sync::mpsc::Sender, time::sleep};

use self::backup::make_backup;

use super::BackupConfig;

#[derive(Deserialize, Clone)]
pub struct DockerPostgresBackupConfig {
    pub folder_name: String,
    pub backup_configs: Vec<BackupConfig>,
    pub docker_container: String,
    pub postgres_user: String,
    pub postgres_database: String,
}

impl DockerPostgresBackupConfig {
    pub async fn spawn_tasks(
        &self,
        age_cert: age::x25519::Recipient,
        history: History,
        history_writer: Sender<ChannelData>,
        tls_client: TlsClient,
    ) {
        // for each backup, spawn a task
        for backup in self.backup_configs.iter() {
            let backup = backup.clone(); // Clone values so they can be moved into the task
            let age_cert = age_cert.clone();
            let history_writer = history_writer.clone();
            let history = history.clone();
            let config = self.clone();
            let tls_client = tls_client.clone();
            let folder_name = self.folder_name.clone();

            tokio::spawn(async move {
                let should_make_backup = match history
                    .should_make_backup(
                        &config.folder_name,
                        &backup.folder_name,
                        backup.backup_interval,
                    )
                    .await
                {
                    Ok(v) => v,
                    Err(error) => {
                        error!(
                            "[[cs]{}/{}[ce]][br]ShouldMakeBackupError[br]{}",
                            &folder_name, &backup.folder_name, error
                        );
                        panic!("ShouldMakeBackupError\n{0}", error)
                    }
                };

                if should_make_backup {
                    Self::try_make_backup(
                        &config,
                        &backup,
                        &folder_name,
                        &age_cert,
                        &history_writer,
                        &tls_client,
                    )
                    .await;
                }

                loop {
                    sleep(backup.backup_interval).await;
                    Self::try_make_backup(
                        &config,
                        &backup,
                        &folder_name,
                        &age_cert,
                        &history_writer,
                        &tls_client,
                    )
                    .await;
                }
            });
        }
    }

    async fn try_make_backup(
        config: &DockerPostgresBackupConfig,
        backup: &BackupConfig,
        folder_name: &String,
        age_cert: &Recipient,
        history_writer: &Sender<ChannelData>,
        tls_client: &TlsClient,
    ) {
        let mut attempt = 0.0;
        loop {
            match make_backup(&config, &backup, &age_cert, &history_writer, &tls_client).await {
                Ok(_) => break,
                Err(error) => {
                    attempt += 1.0;
                    error!(
                        "[[cs]{}/{}[ce]][br]MakeBackupError({})[br]{}",
                        &folder_name, &backup.folder_name, attempt, error
                    );
                    // sigmoid function that flattens at 20 after 15 attempts
                    let backoff_multiplier = 20.0 / (1.0 + E.powf(-0.7 * (attempt - 4.0)));
                    sleep(Duration::from_secs_f32(60.0 * backoff_multiplier)).await;
                }
            }
        }
    }

    pub fn get_names(&self) -> (String, Vec<String>) {
        let backup_folder_names: Vec<String> = self
            .backup_configs
            .iter()
            .map(|backup| backup.folder_name.clone())
            .collect();
        (self.folder_name.clone(), backup_folder_names)
    }
}
