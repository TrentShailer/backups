#![windows_subsystem = "windows"]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cleanup;
mod config;
mod logger;
mod socket;

use cleanup::spawn_cleanup_tasks;
use config::load_config;
use log::{error, info};
use notify_rust::Notification;
use tokio::io::AsyncWriteExt;

use crate::{
    logger::format_message_short,
    socket::{create_socket, handle_connection},
};

#[tokio::main]
async fn main() {
    logger::init_fern().unwrap();

    let (tls_config, program_config) = match load_config() {
        Ok(config) => config,
        Err(error) => {
            error!("LoadConfigError[br]{}", error);
            panic!("Error when loading config");
        }
    };

    let (listener, acceptor) = match create_socket(tls_config).await {
        Ok(val) => val,
        Err(error) => {
            error!("CreateSocketError[br]{}", error);
            panic!("Failed to create socket.");
        }
    };

    spawn_cleanup_tasks(program_config.clone());

    loop {
        let (stream, client_address) = match listener.accept().await {
            Ok(value) => value,
            Err(error) => {
                error!("ListenerAcceptError[br]{}", error);
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let config = program_config.clone();

        tokio::spawn(async move {
            let mut stream = match acceptor.accept(stream).await {
                Ok(v) => v,
                Err(error) => {
                    error!("TlsAcceptorError[br]{}", error);
                    return;
                }
            };

            info!("({}) Client connected", client_address);

            match handle_connection(&mut stream, config, &client_address).await {
                Ok(_) => {
                    info!("({}) Backup successful", client_address);
                    if let Err(error) = stream.write_all(b"success").await {
                        error!("WriteSuccessError[br]{}", error);
                    }

                    if let Err(error) = stream.shutdown().await {
                        error!("ShutdownClientError[br]{}", error);
                    }
                }
                Err(error) => {
                    error!("HandleConnectionError[br]{}", error);
                    let message = format!("error: {}", error);

                    if let Err(error) = stream.write_all(message.as_bytes()).await {
                        error!("WriteErrorError[br]{}", error);
                    }

                    if let Err(error) = stream.shutdown().await {
                        error!("ShutdownClientError[br]{}", error);
                    };

                    let error_body = format_message_short(&error.to_string());

                    if let Err(error) = Notification::new()
                        .summary("Backups Server Error")
                        .body(error_body.as_str())
                        .show()
                    {
                        error!("ShowNotificationError[br]{}", error);
                    }
                }
            }
        });
    }
}
