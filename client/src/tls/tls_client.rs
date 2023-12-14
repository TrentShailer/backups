use std::{io, sync::Arc};

use blake3::Hash;
use log::info;
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

    pub async fn upload_file(&self, payload: Payload) -> Result<(), UploadError> {
        let stream = TcpStream::connect((self.address.clone(), self.port))
            .await
            .map_err(|e| UploadError::TcpConnectError(e))?;

        let mut stream = self
            .connector
            .connect(self.domain.clone(), stream)
            .await
            .map_err(|e| UploadError::TlsConnectError(e))?;

        // send payload
        let payload =
            toml::to_string(&payload).map_err(|e| UploadError::SerializeFileConfigError(e))?;

        loop {
            stream
                .write_all(payload.as_bytes())
                .await
                .map_err(|e| UploadError::SendFileError(e))?;
            stream
                .flush()
                .await
                .map_err(|e| UploadError::SendFileError(e))?;

            let mut response = String::new();
            if let Err(error) = stream.read_to_string(&mut response).await {
                return Err(UploadError::ReadResponseError(error));
            }

            if response == String::from("success") {
                return Ok(());
            }

            if response != String::from("retry") {
                return Err(UploadError::ServerError(response));
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
    #[error("ClientConfigError\n{0}")]
    ClientConfigError(#[source] rustls::Error),
    #[error("DomainError('{0}')\n{1}")]
    DomainError(String, #[source] InvalidDnsNameError),
}
#[derive(Debug, Error)]
pub enum UploadError {
    #[error("TcpConnectError\n{0}")]
    TcpConnectError(#[source] io::Error),
    #[error("TlsConnectError\n{0}")]
    TlsConnectError(#[source] io::Error),
    #[error("SerializeFileConfigError\n{0}")]
    SerializeFileConfigError(#[source] toml::ser::Error),
    #[error("ReadResponseError\n{0}")]
    ReadResponseError(#[source] io::Error),
    #[error("ServerError\n{0}")]
    ServerError(String),
    #[error("SendFileError\n{0}")]
    SendFileError(#[source] io::Error),
}
