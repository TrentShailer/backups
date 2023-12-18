mod backups;
mod config;
mod logger;
mod tls;

use crate::{
    backups::{
        backup_history::{self, load_backup_history},
        backup_types::BackupTypes,
    },
    tls::TlsClient,
};
use config::Config;
use log::error;
use logger::init_fern;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_fern().unwrap();

    let config = match Config::load() {
        Ok(v) => v,
        Err(error) => {
            error!("ConfigLoadError[br]{}", error);
            panic!("ConfigLoadError\n{}", error);
        }
    };

    let mut history = match load_backup_history(&config.program_config) {
        Ok(v) => v,
        Err(error) => {
            error!("LoadBackupHistoryError[br]{}", error);
            panic!("LoadBackupHistoryError\n{}", error);
        }
    };

    let (backup_tx, mut backup_rx) = channel::<backup_history::ChannelData>(10);

    let tls_client = match TlsClient::new(config.tls_config).await {
        Ok(v) => v,
        Err(error) => {
            error!("NewTlsClientError[br]{}", error);
            panic!("NewTlsClientError\n{}", error);
        }
    };

    for service in config.program_config.service_config.iter() {
        match service {
            BackupTypes::DockerPostgres {
                config: backup_config,
            } => {
                let backup_config = backup_config.clone();
                let history = history.clone();
                let backup_tx = backup_tx.clone();
                let tls_client = tls_client.clone();

                tokio::spawn(async move {
                    backup_config
                        .spawn_tasks(history, backup_tx, tls_client)
                        .await;
                });
            }
        };
    }

    let history_manager = tokio::spawn(async move {
        while let Some(data) = backup_rx.recv().await {
            if let Err(error) = history.update_history(data) {
                error!("UpdateHistoryError[br]{}", error);
                continue;
            }
            if let Err(error) = history.save_async().await {
                error!("SaveHistoryError[br]{}", error);
            }
        }
    });

    if let Err(error) = history_manager.await {
        error!("AwaitHistoryManagerError[br]{}", error);
        panic!("AwaitHistoryManagerError\n{}", error);
    };

    Ok(())
}
