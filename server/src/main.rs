#![windows_subsystem = "windows"]

use std::{fs, sync::Arc};

use log::{error, info};
use notify_rust::Notification;
use owo_colors::OwoColorize;
use rustls::server::WebPkiClientVerifier;
use shared::load_certificates;
use tokio::{io::AsyncWriteExt, net::TcpListener, signal};
use tokio_rustls::TlsAcceptor;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{handler::handler, server_config::ServerConfig};

mod cleanup;
mod handler;
mod logger;
mod server_config;

const CONFIG_PATH: &str = "./config.toml";
const BACKUP_PATH: &str = "./backups";

#[tokio::main]
pub async fn main() {
    logger::init_fern().unwrap();

    let config_contents = match fs::read_to_string(CONFIG_PATH) {
        Ok(v) => v,
        Err(e) => return error!("Failed to read config: {}", e),
    };

    let config: ServerConfig = match toml::from_str(&config_contents) {
        Ok(v) => v,
        Err(e) => return error!("Failed to parse config: {}", e),
    };

    let certificates = match load_certificates(
        &config.root_ca_path,
        &config.certificate_path,
        &config.key_path,
    ) {
        Ok(v) => v,
        Err(e) => return error!("Failed to load certificates: {}", e),
    };

    let client_cert_verifier =
        match WebPkiClientVerifier::builder(Arc::new(certificates.root_cert_store)).build() {
            Ok(v) => v,
            Err(e) => return error!("Failed to create client verifier:\n{}", e),
        };

    let tls_config = match rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certificates.certificates, certificates.key)
    {
        Ok(v) => v,
        Err(e) => return error!("Failed to create tls config:\n{}", e),
    };

    let tls = TlsAcceptor::from(Arc::new(tls_config));

    let listener = match TcpListener::bind(&config.socket_address).await {
        Ok(v) => v,
        Err(e) => return error!("Failed bind tcp listener:\n{}", e),
    };

    info!("Listening on {}", config.socket_address);

    let tracker = TaskTracker::new();
    let cancel_token = CancellationToken::new();

    let inner_tracker = tracker.clone();
    let inner_cancel_token = cancel_token.clone();
    tracker.spawn(async move {
        'main: loop {
            let stream = tokio::select! {
                _ = inner_cancel_token.cancelled() => {
                    break 'main;
                }
                stream = listener.accept() => stream
            };

            let (stream, peer_addr) = stream.unwrap();
            let tls = tls.clone();

            inner_tracker.spawn(async move {
                let mut stream = tls.accept(stream).await.unwrap();
                info!("Client connected: {}", peer_addr);

                let mut attempt = 0;

                while attempt < 5 {
                    let result = handler(&mut stream).await;
                    if result.is_ok() {
                        break;
                    }

                    let error = result.err().unwrap();

                    error!(
                        "[{}]\nFailed handling connection: {}",
                        peer_addr.red(),
                        error
                    );

                    if let Err(e) = stream.write_all("retr".as_bytes()).await {
                        error!("[{}]\nFailed sending retry:\n{}", peer_addr.red(), e);
                    }
                    attempt += 1;
                }

                if let Err(e) = stream.write_all("exit".as_bytes()).await {
                    error!("[{}]\nFailed sending exit:\n{}", peer_addr.red(), e);
                }

                if attempt == 5 {
                    if let Err(e) = Notification::new().summary("Backups server error").show() {
                        error!("Failed to show notification:\n{}", e);
                    }
                }
            });
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {
            cancel_token.cancel();
            tracker.close();
            tracker.wait().await;
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal:\n{}", err);
        }
    }
}
