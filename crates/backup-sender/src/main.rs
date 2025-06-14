//! # backup-sender
//!

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::time::Duration;
use std::{fs, path::PathBuf, thread::sleep};

use backup_sender::{config::Config, context::Context, history::History, source::BackupSource};
use shared::{Failure, init_logger};
use tracing::{error, info};

fn main() {
    let _logger = init_logger();

    // Initialize config if args include 'init'.
    if std::env::args().any(|arg| arg.eq("init")) {
        let config = Config::default();
        let contents =
            toml::to_string_pretty(&config).or_log_and_panic("Could not serialize config file");
        fs::write("sender-config.toml", contents).or_log_and_panic("Could not create config file");
        return;
    }

    // Load config
    let config = Config::load_toml(PathBuf::from("./sender-config.toml"))
        .or_log_and_panic("Could not load config");

    // Load history
    let mut history =
        History::load_or_create_file().or_log_and_panic("Could not load or create history");

    loop {
        for source in &config.sources {
            for cadence in source.cadence() {
                let context = Context {
                    service_name: source.service_name(),
                    cadence: *cadence,
                };

                if !history.needs_backup(source.service_name(), *cadence) {
                    continue;
                }

                info!("{context}Making backup");

                let backup = match source.get_backup(*cadence) {
                    Ok(backup) => backup,
                    Err(error) => {
                        error!("{context}Failed to get backup: {error}");
                        continue;
                    }
                };
                let metadata = backup.metadata;
                info!("{context}Got backup");

                if let Err(error) = config.endpoint.send_backup(backup) {
                    error!("{context}Failed to send backup: {error}");
                    continue;
                }
                info!("{context}Sent backup");

                if let Err(error) = history.update(source.service_name(), *cadence) {
                    error!("{context}Could not update history: {error}");
                    continue;
                }

                source.cleanup(metadata);
            }
        }

        sleep(Duration::from_secs(60 * 5));
    }
}
