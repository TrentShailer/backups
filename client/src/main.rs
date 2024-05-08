mod backup;
mod endpoint;
mod history;
mod logger;
mod scheduler_config;
mod service;

use std::{fs, thread::sleep, time::Duration};

use crate::{history::History, scheduler_config::SchedulerConfig};

use backup::Backup;
use error_trace::{ErrorTrace, ResultExt};
use log::{error, info};
use scheduler_config::BackupName;

const CONFIG_PATH: &str = "./config.toml";

fn main() {
    logger::init_fern().unwrap();

    if let Err(e) = client().track() {
        error!("{}", e.to_string());
    }
}

fn client() -> Result<(), ErrorTrace> {
    let config_contents = fs::read_to_string(CONFIG_PATH).context("Read config")?;
    let config: SchedulerConfig = toml::from_str(&config_contents).context("Parse config")?;
    let mut history = History::init().context("Load history")?;

    let mut backups: Vec<Backup> = Vec::new();

    for endpoint in config.endpoints.iter() {
        for service in config.services.iter() {
            for backup in service.backups.iter() {
                let name =
                    BackupName::new(endpoint.name(), &service.service_name, &backup.backup_name);

                let endpoint = &endpoint;
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

    loop {
        for backup in backups.iter() {
            match backup
                .maybe_make_backup(&mut history)
                .with_context(|| format!("Make backup {}", backup.name.to_string()))
            {
                Ok(made_backup) => {
                    if made_backup {
                        info!("Made backup {}", backup.name.to_string())
                    }
                }
                Err(e) => {
                    error!("{}", e.to_string());
                }
            }
        }

        sleep(Duration::from_secs(60 * 5));
    }
}
