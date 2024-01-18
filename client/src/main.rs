mod backup_client;
mod backup_config;
mod history;
mod load_certificates;
mod logger;
mod scheduler_config;

use std::{
    fs,
    sync::Arc,
    time::{Duration, SystemTime},
};

use crate::{
    backup_config::BackupConfig, history::History, load_certificates::load_certificates,
    scheduler_config::SchedulerConfig,
};
use backup_client::make_backup;
use futures_rustls::{rustls::ClientConfig, TlsConnector};
use log::error;
use owo_colors::OwoColorize;
use smol::{future, lock::Mutex, Executor, Task, Timer};

const CONFIG_PATH: &str = "./config.toml";

fn main() {
    logger::init_fern().unwrap();

    let config_contents = match fs::read_to_string(CONFIG_PATH) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to read config: {}", e);
            return;
        }
    };

    let config: SchedulerConfig = match toml::from_str(&config_contents) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return;
        }
    };

    let (certs, key, root_ca, domain) = match load_certificates(&config) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load certificates: {}", e);
            return;
        }
    };

    let history = match History::init() {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load history: {}", e);
            return;
        }
    };

    let history = Arc::new(Mutex::new(history));

    let ex = Executor::new();
    let mut tasks: Vec<Task<()>> = Vec::new();

    for service in config.services.as_slice() {
        for backup in service.backups.as_slice() {
            let task = match create_backup_task(
                &config,
                service,
                backup,
                &certs,
                &key,
                &root_ca,
                &domain,
                &ex,
                history.clone(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "[{}] Failed to create backup task:\n{}",
                        format!("{}/{}", service.service_name, backup.backup_name).red(),
                        e
                    );
                    return;
                }
            };
            tasks.push(task);
        }
    }

    future::block_on(ex.run(future::pending::<()>()));
    unreachable!();
}

fn create_backup_task(
    config: &SchedulerConfig,
    service: &scheduler_config::SchedulerService,
    backup: &scheduler_config::SchedulerBackup,
    certs: &Vec<rustls_pki_types::CertificateDer<'static>>,
    key: &rustls_pki_types::PrivateKeyDer<'static>,
    root_ca: &futures_rustls::rustls::RootCertStore,
    domain: &rustls_pki_types::ServerName<'static>,
    ex: &Executor<'static>,
    history: Arc<Mutex<History>>,
) -> Result<Task<()>, futures_rustls::rustls::Error> {
    let sleep_duration = Duration::from_secs(backup.interval);
    let client_config = BackupConfig::from_scheduler(config, &service, &backup);
    let certs = certs.clone();
    let key = key.clone_key();
    let root_ca = root_ca.clone();
    let domain = domain.clone();

    let tls_config = ClientConfig::builder()
        .with_root_certificates(root_ca)
        .with_client_auth_cert(certs, key)?;

    Ok(ex.spawn(async move {
        let connector = TlsConnector::from(Arc::new(tls_config));

        let mut guard = history.lock().await;

        let last_backed_up =
            guard.last_backed_up(&client_config.service_name, &client_config.backup_name);

        let duration_since = match SystemTime::now().duration_since(last_backed_up) {
            Ok(v) => v,
            Err(e) => e.duration(),
        };

        if duration_since > sleep_duration {
            let backup_result = make_backup(&client_config, &connector, domain.clone()).await;
            if let Err(e) = backup_result {
                error!(
                    "[{}]\nFailed to make backup: {}",
                    format!(
                        "{}::{}",
                        client_config.service_name, client_config.backup_name
                    )
                    .red(),
                    e
                );
            } else {
                if let Err(e) = guard
                    .update(&client_config.service_name, &client_config.backup_name)
                    .await
                {
                    error!(
                        "[{}]\nFailed update history: {}",
                        format!(
                            "{}::{}",
                            client_config.service_name, client_config.backup_name
                        )
                        .red(),
                        e
                    );
                }
            }
        }

        drop(guard);

        loop {
            Timer::after(sleep_duration).await;

            let backup_result = make_backup(&client_config, &connector, domain.clone()).await;

            if let Err(e) = backup_result {
                error!(
                    "[{}]\nFailed to make backup: {}",
                    format!(
                        "{}::{}",
                        client_config.service_name, client_config.backup_name
                    )
                    .red(),
                    e
                );
            } else {
                let mut guard = history.lock().await;

                if let Err(e) = guard
                    .update(&client_config.service_name, &client_config.backup_name)
                    .await
                {
                    error!(
                        "[{}]\nFailed update history: {}",
                        format!(
                            "{}::{}",
                            client_config.service_name, client_config.backup_name
                        )
                        .red(),
                        e
                    );
                }

                drop(guard);
            }
        }
    }))
}
