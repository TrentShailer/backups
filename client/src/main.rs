mod backups;
mod config;
mod tls;

use crate::backups::backup_history::{self, load_backup_history};
use crate::backups::backup_types::BackupTypes;
use crate::tls::TlsClient;
use anyhow::bail;
use config::Config;
use std::io;
use tokio::sync::mpsc::channel;
use tracing::error;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily("logs", "log");
    let (writer, _guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .pretty()
                .compact()
                .with_thread_names(true)
                .with_file(false)
                .with_line_number(false)
                .with_writer(io::stdout),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .pretty()
                .compact()
                .with_thread_names(true)
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

    // TODO update error handling below
    let mut history = match load_backup_history(&config) {
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
            error!("{}", error);
            panic!("Failed to create tls client");
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
                error!("Failed to update history: {}", error);
                continue;
            }
            if let Err(error) = history.save_async().await {
                error!("Failed to save history: {}", error);
            }
        }
    });

    if let Err(error) = history_manager.await {
        error!("{}", error);
        panic!("Failed to await history manager");
    };

    Ok(())
}
