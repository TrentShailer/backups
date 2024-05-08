#![windows_subsystem = "windows"]

use std::{
    fs,
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::Arc,
};

use error_trace::{ErrorTrace, ResultExt};
use log::{error, info};
use notify_rust::Notification;
use rustls::{
    server::{Acceptor, StoresServerSessions, WebPkiClientVerifier},
    Stream,
};
use shared::load_certificates;

use crate::{
    handler::handler,
    server_config::{Blocklist, ServerConfig},
};

mod cleanup;
mod handler;
mod logger;
mod server_config;

const CONFIG_PATH: &str = "./config.toml";
const BACKUP_PATH: &str = "./backups";
pub const BLOCKLIST_PATH: &str = "./blocklist.toml";

pub fn main() {
    logger::init_fern().unwrap();

    if let Err(e) = server().track() {
        error!("{}", e.to_string());
    };
}

fn server() -> Result<(), ErrorTrace> {
    let config_contents = fs::read_to_string(CONFIG_PATH).context("Read config")?;
    let config: ServerConfig = toml::from_str(&config_contents).context("Parse config")?;

    let mut blocklist: Blocklist = if !Path::new(BLOCKLIST_PATH).exists() {
        let blocklist = Blocklist::new();
        blocklist.save().track()?;
        blocklist
    } else {
        let blocklist_contents = fs::read_to_string(BLOCKLIST_PATH).context("Read blocklist")?;
        toml::from_str(&blocklist_contents).context("Parse blocklist")?
    };

    let certificates = load_certificates(
        &config.root_ca_path,
        &config.certificate_path,
        &config.key_path,
    )
    .context("Loading certificates")?;

    let client_cert_verifier =
        WebPkiClientVerifier::builder(Arc::new(certificates.root_cert_store))
            .build()
            .track()?;

    let mut tls_config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certificates.certificates, certificates.key)
        .track()?;

    tls_config.session_storage = Arc::new(NoServerSessionStorage {});

    let tls_config = Arc::new(tls_config);

    let listener = TcpListener::bind(&config.socket_address).track()?;

    info!("Listening on {}", config.socket_address);

    loop {
        let (mut stream, peer_addr) = match listener.accept().track() {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e.to_string());
                Notification::new()
                    .summary("Backups server error")
                    .show()
                    .expect("Failed to show notificaition, should never return Err");
                continue;
            }
        };

        if blocklist.blocked_ips.contains(&peer_addr) {
            info!("Pinged by blocked ip: {}", peer_addr);
            continue;
        }

        if let Err(e) = handle_connection(&mut stream, peer_addr, Arc::clone(&tls_config))
            .with_context(|| format!("Handle connection {}", peer_addr))
        {
            // On client fail, client should get blacklisted
            blocklist.blocked_ips.push(peer_addr);
            if let Err(e) = blocklist.save().context("Save blocklist") {
                error!("{}", e.to_string());
                Notification::new()
                    .summary("Backups server error")
                    .show()
                    .expect("Failed to show notificaition, should never return Err");
            }

            error!("{}", e.to_string());
            Notification::new()
                .summary("Backups server error")
                .show()
                .expect("Failed to show notificaition, should never return Err");
        }
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    peer_addr: SocketAddr,
    tls_config: Arc<rustls::ServerConfig>,
) -> Result<(), ErrorTrace> {
    let mut acceptor = Acceptor::default();

    // consume client hello
    let accepted = loop {
        acceptor.read_tls(stream).track()?;

        if let Some(accepted) = acceptor.accept().track()? {
            break accepted;
        }
    };

    let mut connection = accepted.into_connection(tls_config).track()?;
    let mut stream = Stream::new(&mut connection, stream);

    info!("Client connected: {}", peer_addr);

    let result = handler(&mut stream).context("Handling backup");

    stream.conn.complete_io(stream.sock).track()?;

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
