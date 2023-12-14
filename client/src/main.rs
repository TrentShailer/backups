mod backups;
mod config;
mod tls;

use crate::backups::backup_history::{self, load_backup_history};
use crate::backups::backup_types::BackupTypes;
use crate::tls::TlsClient;
use config::Config;
use std::io;
use tokio::sync::mpsc::channel;
use tokio::task_local;
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("logs", "log");
    let (writer, _guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .pretty()
                .with_file(false)
                .with_line_number(false)
                .with_writer(io::stdout),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .pretty()
                .with_file(false)
                .with_line_number(false)
                .with_writer(writer),
        );

    tracing::subscriber::set_global_default(collector)
        .expect("Failed to create tracing subscriber");

    let config = match Config::load() {
        Ok(v) => v,
        Err(error) => {
            error!("ConfigLoadError -> {}", error);
            panic!("ConfigLoadError -> {}", error);
        }
    };

    let mut history = match load_backup_history(&config.program_config) {
        Ok(v) => v,
        Err(error) => {
            error!("LoadBackupHistoryError -> {}", error);
            panic!("LoadBackupHistoryError -> {}", error);
        }
    };

    let (backup_tx, mut backup_rx) = channel::<backup_history::ChannelData>(10);

    let tls_client = match TlsClient::new(config.tls_config).await {
        Ok(v) => v,
        Err(error) => {
            error!("NewTlsClientError -> {}", error);
            panic!("NewTlsClientError -> {}", error);
        }
    };

    for service in config.program_config.service_config.iter() {
        match service {
            BackupTypes::DockerPostgres {
                config: backup_config,
            } => {
                let backup_config = backup_config.clone();
                let history = history.clone();
                let recipiant = config.program_config.recipiant.clone();
                let backup_tx = backup_tx.clone();
                let tls_client = tls_client.clone();

                tokio::spawn(async move {
                    backup_config
                        .spawn_tasks(recipiant, history, backup_tx, tls_client)
                        .await;
                });
            }
        };
    }

    let history_manager = tokio::spawn(async move {
        while let Some(data) = backup_rx.recv().await {
            if let Err(error) = history.update_history(data) {
                error!("UpdateHistoryError -> {}", error);
                continue;
            }
            if let Err(error) = history.save_async().await {
                error!("SaveHistoryError -> {}", error);
            }
        }
    });

    if let Err(error) = history_manager.await {
        error!("AwaitHistoryManagerError -> {}", error);
        panic!("AwaitHistoryManagerError -> {}", error);
    };

    Ok(())
}
