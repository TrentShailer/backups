//! # backup-sender
//!

use core::time::Duration;
use std::{fs, path::PathBuf, thread::sleep};

use backup_sender::{
    config::Config,
    context::Context,
    history::History,
    source::{BackupSource, Source},
};
use mimalloc::MiMalloc;
use shared::{Failure, init_logger};
use tracing::error;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    let _logger = init_logger();

    // Initialize config if args include 'init'.
    if std::env::args().any(|arg| arg.eq("init")) {
        let config = Config::default();
        let contents =
            toml::to_string_pretty(&config).or_log_and_panic("Could not serialize config file");
        fs::write("config.toml", contents).or_log_and_panic("Could not create config file");
        return;
    }

    // Load config
    let config =
        Config::load_toml(PathBuf::from("./config.toml")).or_log_and_panic("Could not load config");

    // Load history
    let mut history =
        History::load_or_create_file().or_log_and_panic("Could not load or create history");

    loop {
        for source in &config.sources {
            for cadance in source.cadance() {
                let context = Context {
                    service_name: source.service_name(),
                    cadance: *cadance,
                };

                if !history.needs_backup(source.service_name(), *cadance) {
                    continue;
                }

                let backup = match source {
                    Source::DockerPostgres(docker_postgres) => {
                        match docker_postgres.get_backup(*cadance) {
                            Ok(backup) => backup,
                            Err(error) => {
                                error!("{context}Failed to get backup: {error}");
                                continue;
                            }
                        }
                    }
                };

                if let Err(error) = config.endpoint.send_backup(backup) {
                    error!("{context}Failed to send backup: {error}");
                    continue;
                };

                if let Err(error) = history.update(source.service_name(), *cadance) {
                    error!("{context}Could not update history: {error}");
                    continue;
                }
            }
        }

        sleep(Duration::from_secs(60 * 5));
    }
}
