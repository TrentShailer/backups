//! # Backups reciever
//! The webserver that receives backups from a sender.
//!

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::{fs, path::PathBuf};

use backup_receiver::{Config, Receiver};
use mimalloc::MiMalloc;

use shared::init_logger;
use tracing::{error, info};

fn main() {
    let _logger = init_logger().unwrap();

    // Initialize config if args include 'init'.
    if std::env::args().any(|arg| arg.eq("init")) {
        let config = Config::default();
        let contents = toml::to_string_pretty(&config).unwrap();
        fs::write("config.toml", contents).expect("Should be able to write to config.toml");
        return;
    }

    // Load config
    let config = match Config::load_toml(PathBuf::from("./config.toml")) {
        Ok(config) => config,
        Err(error) => {
            error!("Could not load config: {error}");
            return;
        }
    };
    let address = config.socket_address;

    // Create receiver
    let mut receiver = match Receiver::new(config) {
        Ok(receiver) => receiver,
        Err(error) => {
            error!("Could not create receiver: {error}");
            return;
        }
    };

    info!("Listening on: {address}");

    loop {
        receiver.accept_and_handle_client();
    }
}
