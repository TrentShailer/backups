use std::{io::Write, net::TcpStream, path::PathBuf, sync::Arc};

use anyhow::Context;
use chrono::Local;
use rustls::{ClientConfig, ClientConnection, Stream};
use serde::{Deserialize, Serialize};
use shared::{load_certificates, TlsPayload};

use crate::scheduler_config::BackupName;

use super::MakeBackup;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsServer {
    pub socket_address: String,
    pub socket_port: u16,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub root_ca_path: PathBuf,
}

impl MakeBackup for TlsServer {
    fn make_backup(&self, name: &BackupName, max_files: usize, file: &[u8]) -> anyhow::Result<()> {
        let file_hash = blake3::hash(file);
        let payload = TlsPayload {
            file_size: file.len(),
            file_hash,
            file_name: Local::now().format("%Y-%m-%d_%H-%M-%S.backup").to_string(),
            service_name: name.service_name.clone(),
            backup_name: name.backup_name.clone(),
            max_files,
        };

        let payload_string = toml::to_string(&payload).context("Failed to serialize payload")?;

        let certs = load_certificates(&self.root_ca_path, &self.cert_path, &self.key_path)
            .context("Failed to load certificates")?;

        let tls_config = ClientConfig::builder()
            .with_root_certificates(certs.root_cert_store)
            .with_client_auth_cert(certs.certificates, certs.key)
            .context("Failed to create client config")?;

        let mut socket = TcpStream::connect((self.socket_address.clone(), self.socket_port))
            .context("Failed to connect to tcp socket")?;

        let server_name = self
            .socket_address
            .clone()
            .try_into()
            .context("Failed to parse server name")?;

        let mut client = ClientConnection::new(Arc::new(tls_config), server_name)
            .context("Failed to create client conneciton")?;

        let mut stream = Stream::new(&mut client, &mut socket);

        stream
            .write_all(&payload_string.len().to_be_bytes())
            .context("Failed to write payload length")?;
        stream.flush().context("Failed to flush stream")?;

        stream
            .write_all(payload_string.as_bytes())
            .context("Failed to write payload")?;
        stream.flush().context("Failed to flush stream")?;

        stream.write_all(file).context("Failed to write file")?;
        stream.flush().context("Failed to flush stream")?;

        stream
            .conn
            .complete_io(stream.sock)
            .context("Failed to complete io")?;

        Ok(())
    }
}
