use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures_rustls::{rustls::ClientConfig, TlsConnector};
use log::error;
use owo_colors::OwoColorize;
use rustls_pki_types::ServerName;
use smol::{lock::RwLock, Executor, Task, Timer};

use crate::{backup_config::BackupConfig, history::History, make_backup::make_backup};

pub fn create_backup_task(
    client_config: BackupConfig,
    sleep_duration: Duration,
    domain: ServerName<'static>,
    tls_config: Arc<ClientConfig>,
    ex: &Executor<'static>,
    history: Arc<RwLock<History>>,
) -> Result<Task<()>, futures_rustls::rustls::Error> {
    Ok(ex.spawn(async move {
        let connector = TlsConnector::from(tls_config);

        if should_create_backup(history.clone(), &client_config, &sleep_duration).await {
            loop {
                if let Err(e) =
                    make_backup(&client_config, &connector, domain.clone(), history.clone()).await
                {
                    error!(
                        "[{}]\nFailed to make backup: {}",
                        format!(
                            "{}::{}",
                            client_config.service_name, client_config.backup_name
                        )
                        .red(),
                        e
                    );
                    Timer::after(Duration::from_secs(60 * 20)).await; // 20 minute delay after failure before retrying
                } else {
                    break;
                }
            }
        }

        loop {
            Timer::after(sleep_duration).await;

            loop {
                if let Err(e) =
                    make_backup(&client_config, &connector, domain.clone(), history.clone()).await
                {
                    error!(
                        "[{}]\nFailed to make backup: {}",
                        format!(
                            "{}::{}",
                            client_config.service_name, client_config.backup_name
                        )
                        .red(),
                        e
                    );
                    Timer::after(Duration::from_secs(60 * 20)).await; // 20 minute delay after failure before retrying
                } else {
                    break;
                }
            }
        }
    }))
}

async fn should_create_backup(
    history: Arc<RwLock<History>>,
    client_config: &BackupConfig,
    interval: &Duration,
) -> bool {
    let history_reader = history.read().await;
    let last_backed_up =
        history_reader.last_backed_up(&client_config.service_name, &client_config.backup_name);
    drop(history_reader);

    let duration_since = match SystemTime::now().duration_since(last_backed_up) {
        Ok(v) => v,
        Err(e) => e.duration(),
    };
    duration_since > *interval
}
