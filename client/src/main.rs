mod backup_config;
mod create_backup_task;
mod history;
mod load_certificates;
mod logger;
mod make_backup;
mod scheduler_config;

use std::{fs, sync::Arc, time::Duration};

use crate::{
    backup_config::BackupConfig, history::History, load_certificates::load_certificates,
    scheduler_config::SchedulerConfig,
};

use futures_rustls::{rustls::ClientConfig, TlsConnector};
use log::error;
use owo_colors::OwoColorize;
use smol::{future, lock::RwLock, Executor, Task};

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

    let certificates = match load_certificates(&config) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load certificates: {}", e);
            return;
        }
    };

    let tls_config = match ClientConfig::builder()
        .with_root_certificates(certificates.root_cert_store)
        .with_client_auth_cert(certificates.certificates, certificates.key)
    {
        Ok(v) => Arc::new(v),
        Err(e) => {
            error!("Failed to create tls config:\n{}", e);
            return;
        }
    };
    let connector = TlsConnector::from(tls_config);

    let history = match History::init() {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load history: {}", e);
            return;
        }
    };

    let history = Arc::new(RwLock::new(history));

    let ex = Executor::new();
    let mut tasks: Vec<Task<()>> = Vec::new();

    for service in config.services.as_slice() {
        for backup in service.backups.as_slice() {
            let sleep_duration = Duration::from_secs(backup.interval);
            let client_config = BackupConfig::from_scheduler(&config, &service, &backup);

            let task = match create_backup_task::create_backup_task(
                client_config,
                sleep_duration,
                certificates.domain.clone(),
                connector.clone(),
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
