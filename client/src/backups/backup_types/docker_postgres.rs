mod backup;

use crate::{
    backups::backup_history::{history::History, ChannelData},
    tls::TlsClient,
};
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

            tokio::spawn(async move {
                if history
                    .should_make_backup(
                        &config.folder_name,
                        &backup.folder_name,
                        backup.backup_interval,
                    )
                    .await
                {
                    if let Err(error) =
                        make_backup(&config, &backup, &age_cert, &history_writer, &tls_client).await
                    {
                        error!("Failed to make backup: {}", error);
                    };
                }

                loop {
                    sleep(backup.backup_interval).await;
                    if let Err(error) =
                        make_backup(&config, &backup, &age_cert, &history_writer, &tls_client).await
                    {
                        error!("Failed to make backup: {}", error);
                    };
                }
            });
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
