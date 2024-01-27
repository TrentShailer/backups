mod backup;
mod endpoint;
mod history;
mod logger;
mod scheduler_config;
mod service;

use std::{fs, sync::Arc, thread::sleep, time::Duration};

use crate::{history::History, scheduler_config::SchedulerConfig};

use anyhow::Context;
use backup::Backup;
use endpoint::Endpoint;
use log::error;
use owo_colors::OwoColorize;
use scheduler_config::{BackupName, SchedulerService};

const CONFIG_PATH: &str = "./config.toml";

fn main() {
    logger::init_fern().unwrap();

    if let Err(e) = client() {
        error!("{:?}", e);
    }
}

fn client() -> anyhow::Result<()> {
    let config_contents = fs::read_to_string(CONFIG_PATH).context("Failed to read config")?;

    let config: SchedulerConfig =
        toml::from_str(&config_contents).context("Failed to parse config")?;

    let mut history = History::init().context("Failed to load history")?;

    let mut backups: Vec<Backup> = Vec::new();

    let endpoints: Box<[Arc<Endpoint>]> = config.endpoints.into_iter().map(Arc::new).collect();
    let services: Box<[Arc<SchedulerService>]> =
        config.services.into_iter().map(Arc::new).collect();

    for endpoint in endpoints.iter() {
        for service in services.iter() {
            for backup in service.backups.iter() {
                let name =
                    BackupName::new(endpoint.name(), &service.service_name, &backup.backup_name);

                let endpoint = Arc::clone(endpoint);
                let service = &service.config;

                backups.push(Backup::new(
                    endpoint,
                    service,
                    name,
                    Duration::from_secs(backup.interval),
                    backup.max_files,
                ))
            }
        }
    }

    println!("{}", backups.len());

    loop {
        for backup in backups.iter() {
            if let Err(e) = backup.maybe_make_backup(&mut history).with_context(|| {
                format!("[{}] Failed to make backup", backup.name.to_string().red())
            }) {
                error!("{:?}", e);
            };
        }

        sleep(Duration::from_secs(60 * 5));
    }
}
