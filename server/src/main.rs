#![windows_subsystem = "windows"]

use std::{
    fs,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};

use anyhow::Context;
use log::{error, info};
use notify_rust::Notification;
use owo_colors::OwoColorize;
use rustls::{
    server::{Acceptor, StoresServerSessions, WebPkiClientVerifier},
    Stream,
};
use shared::load_certificates;

use crate::{handler::handler, server_config::ServerConfig};

mod cleanup;
mod handler;
mod logger;
mod server_config;

const CONFIG_PATH: &str = "./config.toml";
const BACKUP_PATH: &str = "./backups";

pub fn main() {
    logger::init_fern().unwrap();

    if let Err(e) = server() {
        error!("{:?}", e);
    };
}

fn server() -> anyhow::Result<()> {
    let config_contents = fs::read_to_string(CONFIG_PATH).context("Failed to read config")?;

    let config: ServerConfig =
        toml::from_str(&config_contents).context("Failed to parse config")?;

    let certificates = load_certificates(
        &config.root_ca_path,
        &config.certificate_path,
        &config.key_path,
    )
    .context("Failed to load certificates")?;

    let client_cert_verifier =
        WebPkiClientVerifier::builder(Arc::new(certificates.root_cert_store))
            .build()
            .context("Failed to create client verifier")?;

    let mut tls_config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certificates.certificates, certificates.key)
        .context("Failed to create tls config")?;

    tls_config.session_storage = Arc::new(NoServerSessionStorage {});

    let tls_config = Arc::new(tls_config);

    let listener =
        TcpListener::bind(&config.socket_address).context("Failed to bind tcp listener")?;

    info!("Listening on {}", config.socket_address);

    loop {
        let (mut stream, peer_addr) = match listener.accept() {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to accept client:\n{:?}", e);
                continue;
            }
        };

        if let Err(e) = handle_connection(&mut stream, peer_addr, Arc::clone(&tls_config))
            .with_context(|| format!("[{}]\nFailed to handle connection", peer_addr.red()))
        {
            error!("{:?}", e);
            if let Err(e) = Notification::new().summary("Backups server error").show() {
                error!("Failed to show notification:\n{:?}", e);
            };
        };
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    peer_addr: SocketAddr,
    tls_config: Arc<rustls::ServerConfig>,
) -> anyhow::Result<()> {
    let mut acceptor = Acceptor::default();

    // consume client hello
    let accepted = loop {
        acceptor
            .read_tls(stream)
            .context("Failed to read client hello")?;
        if let Some(accepted) = acceptor.accept().context("Failed to accept client_hello")? {
            break accepted;
        }
    };

    let mut connection = accepted
        .into_connection(tls_config)
        .context("Failed to accept connection")?;

    let mut stream = Stream::new(&mut connection, stream);

    info!("Client connected: {}", peer_addr);

    let result = handler(&mut stream).context("Failed to handle connection");

    stream
        .conn
        .complete_io(stream.sock)
        .context("Failed to complete io")?;

    result
}

#[derive(Debug)]
struct NoServerSessionStorage;

impl StoresServerSessions for NoServerSessionStorage {
    fn put(&self, _key: Vec<u8>, _value: Vec<u8>) -> bool {
        true
    }

    fn get(&self, _key: &[u8]) -> Option<Vec<u8>> {
        None
    }

    fn take(&self, _key: &[u8]) -> Option<Vec<u8>> {
        None
    }

    fn can_cache(&self) -> bool {
        false
    }
}
