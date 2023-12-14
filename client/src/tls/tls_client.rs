use std::{io, sync::Arc};

use blake3::Hash;
use rustls_pki_types::{InvalidDnsNameError, ServerName};
use serde::Serialize;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::config::TlsConfig;

#[derive(Clone)]
pub struct TlsClient {
    address: String,
    domain: ServerName<'static>,
    port: u16,
    connector: TlsConnector,
}

impl TlsClient {
    pub async fn new(config: TlsConfig) -> Result<Self, NewClientError> {
        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(config.root_ca)
            .with_client_auth_cert(config.certificate, config.key)
            .map_err(|e| NewClientError::ClientConfigError(e))?;
        let connector = TlsConnector::from(Arc::new(tls_config));

        let domain = ServerName::try_from(config.address.clone())
            .map_err(|e| NewClientError::DomainError(config.address.clone(), e))?
            .to_owned();

        Ok(Self {
            address: config.address,
            port: config.port,
            domain,
            connector,
        })
    }

    pub async fn upload_file(
        &self,
        file_config: OutgoingBackupConfig,
        file: Vec<u8>,
    ) -> Result<(), UploadError> {
        let stream = TcpStream::connect((self.address.clone(), self.port))
            .await
            .map_err(|e| UploadError::TcpConnectError(e))?;

        let mut stream = self
            .connector
            .connect(self.domain.clone(), stream)
            .await
            .map_err(|e| UploadError::TlsConnectError(e))?;

        // send backup config
        let file_config_string =
            toml::to_string(&file_config).map_err(|e| UploadError::SerializeFileConfigError(e))?;

        stream
            .write_all(file_config_string.as_bytes())
            .await
            .map_err(|e| UploadError::SendFileConfigError(e))?;
        stream
            .flush()
            .await
            .map_err(|e| UploadError::SendFileConfigError(e))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .map_err(|e| UploadError::ReadResponseError(e))?;

        if response != String::from("ready") {
            return Err(UploadError::ServerError(response));
        }

        // send file
        loop {
            stream
                .write_all(&file)
                .await
                .map_err(|e| UploadError::SendFileError(e))?;

            stream
                .flush()
                .await
                .map_err(|e| UploadError::SendFileError(e))?;

            let mut response = String::new();
            if let Err(error) = stream.read_to_string(&mut response).await {
                match error.kind() {
                    io::ErrorKind::ConnectionAborted => return Ok(()),
                    _ => return Err(UploadError::ReadResponseError(error)),
                }
            }

            if response != String::from("retry") {
                return Err(UploadError::ServerError(response));
            }
        }
    }
}
#[derive(Serialize)]
pub struct OutgoingBackupConfig {
    pub folder: String,
    pub sub_folder: String,
    pub file_name: String,
    pub file_hash: Hash,
}

#[derive(Debug, Error)]
pub enum NewClientError {
    #[error("ClientConfigError -> {0}")]
    ClientConfigError(#[source] rustls::Error),
    #[error("DomainError('{0}') -> {1}")]
    DomainError(String, #[source] InvalidDnsNameError),
}
#[derive(Debug, Error)]
pub enum UploadError {
    #[error("TcpConnectError -> {0}")]
    TcpConnectError(#[source] io::Error),
    #[error("TlsConnectError -> {0}")]
    TlsConnectError(#[source] io::Error),
    #[error("SerializeFileConfigError -> {0}")]
    SerializeFileConfigError(#[source] toml::ser::Error),
    #[error("SendFileConfigError -> {0}")]
    SendFileConfigError(#[source] io::Error),
    #[error("ReadResponseError -> {0}")]
    ReadResponseError(#[source] io::Error),
    #[error("ServerError -> {0}")]
    ServerError(String),
    #[error("SendFileError -> {0}")]
    SendFileError(#[source] io::Error),
}
