use std::{io::Write, net::TcpStream, path::PathBuf, sync::Arc};

use chrono::Local;
use error_trace::{ErrorTrace, ResultExt};
use rustls::{ClientConfig, ClientConnection, Stream};
use rustls_pki_types::ServerName;
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
    fn make_backup(
        &self,
        name: &BackupName,
        max_files: usize,
        file: &[u8],
    ) -> Result<(), ErrorTrace> {
        let file_hash = blake3::hash(file);
        let payload = TlsPayload {
            file_size: file.len(),
            file_hash,
            file_name: Local::now().format("%Y-%m-%d_%H-%M-%S.backup").to_string(),
            service_name: name.service_name.clone(),
            backup_name: name.backup_name.clone(),
            max_files,
        };

        let payload_string = toml::to_string(&payload).context("Serialize payload")?;

        let certs = load_certificates(&self.root_ca_path, &self.cert_path, &self.key_path)
            .context("Load certificates")?;

        let tls_config = ClientConfig::builder()
            .with_root_certificates(certs.root_cert_store)
            .with_client_auth_cert(certs.certificates, certs.key)
            .context("Create client config")?;

        let mut socket = TcpStream::connect((self.socket_address.clone(), self.socket_port))
            .context("Connect to tcp socket")?;

        let server_name: Result<ServerName, _> = self.socket_address.clone().try_into();
        let server_name = server_name.context("Convert server name")?;

        let mut client = ClientConnection::new(Arc::new(tls_config), server_name)
            .context("Create client conneciton")?;

        let mut stream = Stream::new(&mut client, &mut socket);

        stream
            .write_all(&payload_string.len().to_be_bytes())
            .context("Write payload length")?;
        stream.flush().context("Flush stream")?;

        stream
            .write_all(payload_string.as_bytes())
            .context("Write payload")?;
        stream.flush().context("Flush stream")?;

        stream.write_all(file).context("Write file")?;
        stream.flush().context("Flush stream")?;

        stream
            .conn
            .complete_io(stream.sock)
            .context("Complete io")?;

        Ok(())
    }
}
