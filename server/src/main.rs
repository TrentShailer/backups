mod config;

use std::net::SocketAddr;

use config::load_config;
use log::{error, info, warn};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => warn!("No .env file found"), // Log to info as variables could already be in env
    };

    let config = match load_config() {
        Ok(config) => config,
        Err(error) => {
            error!("{}", error);
            panic!("Error when loading config");
        }
    };

    info!("Address: {0}", config.address);

    let listener = TcpListener::bind(&config.address).await?;

    info!("Listening on address: {0}", config.address);

    loop {
        let (mut socket, client_address) = listener.accept().await?;

        tokio::spawn(async move {
            info!("TCP connection from {}", client_address);
            handle_connection(&mut socket, client_address).await;
            info!("TCP connection from {} closed", client_address);
        });
    }
}

async fn handle_connection(socket: &mut TcpStream, client_address: SocketAddr) {}
