mod cleanup;
mod config;
mod logger;
mod socket;

use cleanup::spawn_cleanup_tasks;
use config::load_config;
use log::info;
use tokio::io::AsyncWriteExt;

use crate::socket::{create_socket, handle_connection};

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let (tls_config, program_config) = match load_config() {
        Ok(config) => config,
        Err(error) => {
            app_error!("LoadConfigError[br]{}", error);
            panic!("Error when loading config");
        }
    };

    let (listener, acceptor) = match create_socket(tls_config).await {
        Ok(val) => val,
        Err(error) => {
            app_error!("CreateSocketError[br]{}", error);
            panic!("Failed to create socket.");
        }
    };

    spawn_cleanup_tasks(program_config.clone());

    loop {
        let (stream, client_address) = match listener.accept().await {
            Ok(value) => value,
            Err(error) => {
                app_error!("ListenerAcceptError[br]{}", error);
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let config = program_config.clone();

        tokio::spawn(async move {
            let mut stream = match acceptor.accept(stream).await {
                Ok(v) => v,
                Err(error) => {
                    app_error!("TlsAcceptorError[br]{}", error);
                    return;
                }
            };

            info!("Client connected: {}", client_address);

            match handle_connection(&mut stream, config).await {
                Ok(_) => {
                    if let Err(error) = stream.write_all(b"success").await {
                        app_error!("WriteSuccessError[br]{}", error);
                    }

                    if let Err(error) = stream.shutdown().await {
                        app_error!("ShutdownClientError[br]{}", error);
                    }
                }
                Err(error) => {
                    app_error!("HandleConnectionError[br]{}", error);
                    let message = format!("error: {}", error);

                    if let Err(error) = stream.write_all(message.as_bytes()).await {
                        app_error!("WriteErrorError[br]{}", error);
                    }

                    if let Err(error) = stream.shutdown().await {
                        app_error!("ShutdownClientError[br]{}", error);
                    };

                    // TODO raise flag to me
                }
            }
        });
    }
}
