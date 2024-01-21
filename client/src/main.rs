mod backup_task;
mod endpoint;
mod history;
mod logger;
mod scheduler_config;
mod service;

use std::{fs, sync::Arc, time::Duration};

use crate::{history::History, scheduler_config::SchedulerConfig};

use backup_task::backup_task;
use endpoint::Endpoint;
use log::error;
use scheduler_config::BackupName;
use tokio::{signal, sync::RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

const CONFIG_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() {
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

    let history = match History::init() {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load history: {}", e);
            return;
        }
    };

    let history = Arc::new(RwLock::new(history));

    let tracker = TaskTracker::new();
    let cancel_token = CancellationToken::new();

    let endpoints: Vec<Arc<Endpoint>> = config
        .endpoints
        .into_iter()
        .map(|endpoint| Arc::new(endpoint))
        .collect();

    for endpoint in endpoints {
        for service in config.services.iter() {
            for backup in service.backups.iter() {
                let name =
                    BackupName::new(endpoint.name(), &service.service_name, &backup.backup_name);
                let service_config = service.config.clone();
                let history = Arc::clone(&history);
                let cancel_token = cancel_token.clone();
                let endpoint = endpoint.clone();

                tracker.spawn(backup_task(
                    endpoint,
                    service_config,
                    name,
                    backup.max_files,
                    Duration::from_secs(backup.interval),
                    history,
                    cancel_token,
                ));
            }
        }
    }

    match signal::ctrl_c().await {
        Ok(()) => {
            cancel_token.cancel();
            tracker.close();
            tracker.wait().await;
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal:\n{}", err);
        }
    }
}
