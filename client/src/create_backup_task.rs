use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures_rustls::{rustls::ClientConfig, TlsConnector};
use log::error;
use owo_colors::OwoColorize;
use smol::{lock::RwLock, Executor, Task, Timer};

use crate::{
    backup_config::BackupConfig,
    history::History,
    load_certificates::Certificates,
    make_backup::make_backup,
    scheduler_config::{SchedulerBackup, SchedulerConfig, SchedulerService},
};

pub fn create_backup_task(
    config: &SchedulerConfig,
    service: &SchedulerService,
    backup: &SchedulerBackup,
    certificates: &Certificates,
    ex: &Executor<'static>,
    history: Arc<RwLock<History>>,
) -> Result<Task<()>, futures_rustls::rustls::Error> {
    let sleep_duration = Duration::from_secs(backup.interval);
    let client_config = BackupConfig::from_scheduler(config, &service, &backup);
    let domain = certificates.domain.clone();

    let tls_config = ClientConfig::builder()
        .with_root_certificates(certificates.root_cert_store.clone())
        .with_client_auth_cert(
            certificates.certificates.clone(),
            certificates.key.clone_key(),
        )?;

    Ok(ex.spawn(async move {
        let connector = TlsConnector::from(Arc::new(tls_config));

        if should_create_backup(history.clone(), &client_config, &sleep_duration).await {
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
            }
        }

        loop {
            Timer::after(sleep_duration).await;

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
