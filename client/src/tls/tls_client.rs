use std::{io, str::Utf8Error, sync::Arc};

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
            .map_err(NewClientError::ClientConfigError)?;
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

    pub async fn upload_file(&self, payload: Payload) -> Result<(), UploadError> {
        let stream = TcpStream::connect((self.address.clone(), self.port))
            .await
            .map_err(UploadError::TcpConnectError)?;

        let mut stream = self
            .connector
            .connect(self.domain.clone(), stream)
            .await
            .map_err(UploadError::TlsConnectError)?;

        // send payload
        let payload = toml::to_string(&payload).map_err(UploadError::SerializeFileConfigError)?;

        loop {
            stream
                .write_all(payload.as_bytes())
                .await
                .map_err(UploadError::SendFileError)?;

            let mut response: Vec<u8> = vec![0; 16384];
            let response_size = match stream.read(&mut response).await {
                Ok(v) => v,
                Err(error) => return Err(UploadError::ReadResponseError(error)),
            };
            let response = std::str::from_utf8(&response[0..response_size])?;

            if response == "success" {
                return Ok(());
            }

            if response != "retry" {
                return Err(UploadError::ServerError(String::from(response)));
            }
        }
    }
}

#[derive(Serialize)]
pub struct Payload {
    pub folder: String,
    pub sub_folder: String,
    pub file_name: String,
    pub file_hash: Hash,
    pub file: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum NewClientError {
    #[error("ClientConfigError[br]{0}")]
    ClientConfigError(#[source] rustls::Error),
    #[error("DomainError('{0}')[br]{1}")]
    DomainError(String, #[source] InvalidDnsNameError),
}
#[derive(Debug, Error)]
pub enum UploadError {
    #[error("TcpConnectError[br]{0}")]
    TcpConnectError(#[source] io::Error),
    #[error("TlsConnectError[br]{0}")]
    TlsConnectError(#[source] io::Error),
    #[error("SerializeFileConfigError[br]{0}")]
    SerializeFileConfigError(#[source] toml::ser::Error),
    #[error("ReadResponseError[br]{0}")]
    ReadResponseError(#[source] io::Error),
    #[error("ConvertResponseError[br]{0}")]
    ConvertResponseError(#[from] Utf8Error),
    #[error("ServerError[br]{0}")]
    ServerError(String),
    #[error("SendFileError[br]{0}")]
    SendFileError(#[source] io::Error),
}
