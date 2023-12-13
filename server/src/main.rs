mod certificate;
mod config;
mod config_types;
mod socket;

use config::load_config;
use log::{error, info};
use tokio::io::AsyncWriteExt;

use crate::socket::{create_socket, handle_connection};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (tls_config, config) = match load_config() {
        Ok(config) => {
            info!("Loaded config");
            config
        }
        Err(error) => {
            error!("{}", error);
            panic!("Error when loading config");
        }
    };

    let (listener, acceptor) = match create_socket(tls_config).await {
        Ok(val) => {
            info!("Created socket");
            val
        }
        Err(error) => {
            error!("{}", error);
            panic!("Failed to create socket.");
        }
    };

    /*
       TODO Spawn file cleanup task
    */

    loop {
        let (stream, client_address) = match listener.accept().await {
            Ok(value) => value,
            Err(error) => {
                error!("{}", error);
                panic!("Failed to accept client");
            }
        };
        let acceptor = acceptor.clone();
        let config = config.clone();

        tokio::spawn(async move {
            let mut stream = match acceptor.accept(stream).await {
                Ok(v) => v,
                Err(error) => {
                    error!("Failed to accept client stream: {}", error);
                    return;
                }
            };

            info!("Client connected: {}", client_address);

            match handle_connection(&mut stream, config).await {
                Ok(_) => {}
                Err(error) => {
                    error!("{}", error);
                    let message = format!("error: {}", error);

                    if let Err(error) = stream.write_all(message.as_bytes()).await {
                        error!("Failed to write error to client: {}", error);
                    }

                    if let Err(error) = stream.shutdown().await {
                        error!("Failed to shutdown client connection: {}", error);
                    };

                    // TODO raise flag to me
                }
            }
        });
    }
}
