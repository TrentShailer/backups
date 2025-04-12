//! # Backup reciever
//! The webserver that receives backups from a sender.
//!

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::{fs, path::PathBuf};

use backup_receiver::{Config, Receiver};
use mimalloc::MiMalloc;

use shared::{Failure, init_logger};
use tracing::info;

fn main() {
    let _logger = init_logger().unwrap();

    // Initialize config if args include 'init'.
    if std::env::args().any(|arg| arg.eq("init")) {
        let config = Config::default();
        let contents =
            toml::to_string_pretty(&config).or_log_and_panic("Could not serialize config file");
        fs::write("receiver-config.toml", contents)
            .or_log_and_panic("Could not create config file");
        return;
    }

    // Load config
    let config = Config::load_toml(PathBuf::from("./receiver-config.toml"))
        .or_log_and_panic("Could not load config");
    let address = config.socket_address;

    // Create receiver
    let mut receiver = Receiver::new(config).or_log_and_panic("Could not create receiver");

    info!("Listening on: {address}");

    loop {
        receiver.accept_and_handle_client();
    }
}
