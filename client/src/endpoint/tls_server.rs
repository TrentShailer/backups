use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use rustls::ClientConfig;
use rustls_pki_types::{InvalidDnsNameError, ServerName};
use serde::{Deserialize, Serialize};
use shared::{load_certificates, LoadCertsError, TlsPayload};
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

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
    type Error = MakeBackupError;

    async fn make_backup(
        &self,
        name: &BackupName,
        max_files: usize,
        file: &[u8],
    ) -> Result<(), Self::Error> {
        let file_hash = blake3::hash(file);
        let payload = TlsPayload {
            file_size: file.len(),
            file_hash,
            file_name: Local::now().format("%Y-%m-%d_%H-%M-%S.backup").to_string(),
            service_name: name.service_name.clone(),
            backup_name: name.backup_name.clone(),
            max_files,
        };

        let payload_string = toml::to_string(&payload)?;

        let certs = load_certificates(&self.root_ca_path, &self.cert_path, &self.key_path)?;

        let tls_config = ClientConfig::builder()
            .with_root_certificates(certs.root_cert_store)
            .with_client_auth_cert(certs.certificates, certs.key)?;

        let connector = TlsConnector::from(Arc::new(tls_config));

        let domain = ServerName::try_from(self.socket_address.clone())?;

        let stream = TcpStream::connect((self.socket_address.clone(), self.socket_port)).await?;
        let mut stream = connector.connect(domain, stream).await?;

        loop {
            stream
                .write_all(&payload_string.len().to_be_bytes())
                .await?;
            stream.flush().await?;
            stream.write_all(payload_string.as_bytes()).await?;
            stream.flush().await?;
            stream.write_all(file).await?;

            let mut response: Vec<u8> = vec![0; 4];
            stream.read_exact(&mut response).await?;
            if response == b"exit" {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum MakeBackupError {
    #[error("LoadCertsError: {0}")]
    LoadCerts(#[from] LoadCertsError),
    #[error("TlsConfigError:\n{0}")]
    TlsConfig(#[from] rustls::Error),
    #[error("IoError:\n{0}")]
    IoError(#[from] io::Error),
    #[error("ServerNameError:\n{0}")]
    ServerName(#[from] InvalidDnsNameError),
    #[error("SerializePayloadError:\n{0}")]
    SerializePayload(#[from] toml::ser::Error),
}
