//! # backup-sender
//!

use core::time::Duration;
use std::{fs, path::PathBuf, thread::sleep};

use backup_sender::{
    config::Config,
    history::{self, History},
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
            // for cadance in source.ca
            //
        }
        //

        sleep(Duration::from_secs(60 * 5));
    }
}
