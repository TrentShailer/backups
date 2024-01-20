// #![windows_subsystem = "windows"]

use std::{fs, sync::Arc};

use futures_rustls::{
    rustls::{self, server::WebPkiClientVerifier},
    TlsAcceptor,
};
use load_certificates::load_certificates;
use log::{error, info};
use notify_rust::Notification;
use owo_colors::OwoColorize;
use smol::{io::AsyncWriteExt, net::TcpListener};

use crate::{handler::handler, server_config::ServerConfig};

mod cleanup;
mod handler;
mod load_certificates;
mod logger;
mod payload;
mod server_config;

const CONFIG_PATH: &str = "./config.toml";
const BACKUP_PATH: &str = "./backups";

pub fn main() {
    logger::init_fern().unwrap();

    let config_contents = match fs::read_to_string(CONFIG_PATH) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to read config: {}", e);
            return;
        }
    };

    let config: ServerConfig = match toml::from_str(&config_contents) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse config: {}", e);
            return;
        }
    };

    let certificates = match load_certificates(&config) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load certificates: {}", e);
            return;
        }
    };

    let client_cert_verifier =
        match WebPkiClientVerifier::builder(Arc::new(certificates.root_cert_store)).build() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to create client verifier:\n{}", e);
                return;
            }
        };

    let tls_config = match rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certificates.certificates, certificates.key)
    {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to create tls config:\n{}", e);
            return;
        }
    };

    smol::block_on(async {
        let tls = TlsAcceptor::from(Arc::new(tls_config));
        let listener = match TcpListener::bind(&config.socket_address).await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed bind tcp listener:\n{}", e);
                return;
            }
        };

        info!("Listening on address: {}", config.socket_address);

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed accept tcp listener:\n{}", e);
                    continue;
                }
            };
            let mut stream = match tls.accept(stream).await {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed accept tls listener:\n{}", e);
                    continue;
                }
            };

            let peer_addr = match stream.get_ref().0.peer_addr() {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed get peer addr:\n{}", e);
                    continue;
                }
            };

            info!("Client connected: {}", peer_addr);

            smol::spawn(async move {
                let mut attempt = 0;

                while attempt < 5 {
                    if let Err(e) = handler(&mut stream).await {
                        error!("[{}]\nFailed handling connection: {}", peer_addr.red(), e);

                        if let Err(e) = stream.write_all("retry".as_bytes()).await {
                            error!("[{}]\nFailed sending retry:\n{}", peer_addr.red(), e);
                        }
                        attempt += 1;
                    } else {
                        break;
                    }
                }

                if let Err(e) = stream.write_all("exit".as_bytes()).await {
                    error!("[{}]\nFailed sending exit:\n{}", peer_addr.red(), e);
                }

                if attempt == 5 {
                    if let Err(e) = Notification::new().summary("Backups server error").show() {
                        error!("Failed to show notification:\n{}", e);
                    }
                }
            })
            .detach();
        }
    })
}
