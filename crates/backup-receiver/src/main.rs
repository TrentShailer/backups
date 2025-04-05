//! # Backups reciever
//! The webserver that receives backups from a sender.
//!

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use core::time::Duration;
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

        let (mut connection, mut stream, peer) = match receiver.accept_blocking(&mut context) {
            Ok(client) => client,
            Err(error) => {
                warn!("{context}Failed to accept mTLS connection: {error}");
                continue;
            }
        };

        let mut stream = Stream::new(&mut connection, &mut stream);

        // Refresh and check rate limit
        if let Some(history) = receiver.history.get_mut(&peer.ip()) {
            history.retain(|backup_time| backup_time.elapsed() < Duration::from_secs(60 * 60));

            if history.len() >= receiver.config.limits.maximum_backups_per_hour {
                warn!("{context}Exceeded rate limit");
                receiver.send_response_and_close(
                    &mut context,
                    &mut stream,
                    Response::ExceededRateLimit,
                );
                continue;
            }
        }

        let metadata = match receiver.read_metadata(&mut context, &mut stream) {
            Ok(metadata) => metadata,
            Err(response) => {
                receiver.send_response_and_close(&mut context, &mut stream, response);
                continue;
            }
        };

        let mut file = match receiver.prepare_backup_file(&mut context, &metadata) {
            Ok(file) => file,
            Err(response) => {
                receiver.send_response_and_close(&mut context, &mut stream, response);
                continue;
            }
        };

        if let Err(response) =
            receiver.read_and_write_payload(&mut context, &mut stream, &metadata, &mut file)
        {
            receiver.send_response_and_close(&mut context, &mut stream, response);
            continue;
        }

        receiver.send_response_and_close(&mut context, &mut stream, Response::Success);

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
