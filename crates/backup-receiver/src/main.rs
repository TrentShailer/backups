//! # Backups reciever
//! The webserver that receives backups from a sender.
//!

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::{fs, path::PathBuf, time::Instant};

use backup_receiver::{Config, ContextLogger, Receiver, cleanup};
use mimalloc::MiMalloc;
use rustls::Stream;
use shared::{Response, init_logger};
use tracing::{error, warn};

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

    // Create receiver
    let mut receiver = match Receiver::new(config) {
        Ok(receiver) => receiver,
        Err(error) => {
            error!("Could not create receiver: {error}");
            return;
        }
    };

    loop {
        let mut context = ContextLogger::default();

        let (mut connection, mut stream, peer) = match receiver.accept_client(&mut context) {
            Ok(client) => client,
            Err(error) => {
                warn!("{context}Failed to accept mTLS connection: {error}");
                continue;
            }
        };

        let mut stream = Stream::new(&mut connection, &mut stream);

        let metadata = match receiver.handle_client(&mut context, &mut stream, peer) {
            Ok(metadata) => {
                receiver.send_response_and_close(&mut context, &mut stream, Response::Success);
                metadata
            }
            Err(response) => {
                receiver.send_response_and_close(&mut context, &mut stream, response);
                continue;
            }
        };

        // Track backup in history
        if let Some(history) = receiver.history.get_mut(&peer.ip()) {
            history.push(Instant::now());
        } else {
            receiver.history.insert(peer.ip(), vec![Instant::now()]);
        }

        // Clean up files
        cleanup(&mut context, &receiver.config, &metadata);
    }
}
