mod config;
mod endpoint;
mod history;
mod logger;
mod service;

use std::{thread::sleep, time::Duration};

use crate::history::History;

use config::{backup::Backup, Config};
use log::{error, info};
use thiserror::Error;

fn main() {
    logger::init_fern().unwrap();

    if let Err(e) = client() {
        error!("{}", e.to_string());
    }
}

fn client() -> Result<(), Error> {
    let config = Config::load()?;
    let mut history = History::load_or_create()?;

    let backups: Vec<Backup> = config
        .backup_pairs
        .iter()
        .flat_map(|pair| pair.as_backups())
        .collect();

    loop {
        for backup in backups.iter() {
            if backup.is_backup_due(&mut history) {
                info!(
                    "Making backup for {}/{}/{}",
                    backup.service_name, backup.endpoint_name, backup.backup_name
                );

                match backup.make_backup(&mut history) {
                    Ok(_) => info!(
                        "Made backup {}/{}/{}",
                        backup.service_name, backup.endpoint_name, backup.backup_name
                    ),

                    Err(e) => error!(
                        "Failed to make backup to {}/{}/{}\n{e}",
                        backup.service_name, backup.endpoint_name, backup.backup_name
                    ),
                }
            }
        }

        sleep(Duration::from_secs(60 * 5));
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to load config:\n{0}")]
    LoadConfig(#[from] config::Error),

    #[error("Failed to load histoy:\n{0}")]
    LoadHistory(#[from] history::Error),
}
