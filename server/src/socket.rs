mod backup_config;
mod handle_file;

use log::{error, info, warn};
use rustls::server::{VerifierBuilderError, WebPkiClientVerifier};
use std::{io, sync::Arc};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    config_types::{Config, TlsConfig},
    socket::backup_config::recieve_backup_config,
};

use self::{backup_config::IncomingBackupConfigError, handle_file::HandleFileError};

#[derive(Debug, Error)]
pub enum CreateSocketError {
    #[error("Failed to build client verifier: {0}")]
    BuildClientVerifierError(#[source] VerifierBuilderError),
    #[error("Failed to build tls config: {0}")]
    BuildTlsConfigError(#[source] rustls::Error),
    #[error("Failed to bind tcp socket: {0}")]
    TcpBindError(#[source] io::Error),
}

pub async fn create_socket(
    config: TlsConfig,
) -> Result<(TcpListener, TlsAcceptor), CreateSocketError> {
    let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(config.root_ca))
        .build()
        .map_err(|error| CreateSocketError::BuildClientVerifierError(error))?;

    let tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(config.certificate_chain, config.private_key)
        .map_err(|error| CreateSocketError::BuildTlsConfigError(error))?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind(&config.socket_address)
        .await
        .map_err(|error| CreateSocketError::TcpBindError(error))?;

    info!("Listening on address: {0}", config.socket_address);

    Ok((listener, tls_acceptor))
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("ReadPayloadError\n{0}")]
    ReadPayloadError(#[source] io::Error),
    // ----
    #[error("Failed to parse backup config")]
    BackupConfigError(#[source] IncomingBackupConfigError),
    #[error("Failed to send ready message to client: {0}")]
    SendReadyError(#[source] io::Error),
    #[error("Failed to send retry message to client: {0}")]
    SendRetryError(#[source] io::Error),
    #[error("Failed to download file: {0}")]
    DownloadError(#[source] HandleFileError),
    #[error("Failed to download file, reached maximum retries")]
    RetryError,
}

const FILE_RETRIES: u8 = 5;

pub async fn handle_connection(
    stream: &mut tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    server_config: Config,
) -> Result<(), ConnectionError> {
    // TODO rewrite everything
    // try recieve payload
    // 		try parse payload
    //		try check with own config
    //		try decrypt
    //		try save
    //		try check hash
    // respond with success, retry, or an error
    // first implement without retires

    // read payload
    let mut buffer = vec![0; 1024];
    let mut response = String::new();
    if let Err(error) = stream.read_to_string(&mut response).await {
        return Err(ConnectionError::ReadPayloadError(error));
    }

    info!("{}", response);

    let backup_config = recieve_backup_config(&server_config, stream)
        .await
        .map_err(|e| ConnectionError::BackupConfigError(e))?;

    stream
        .write_all(b"ready")
        .await
        .map_err(|e| ConnectionError::SendReadyError(e))?;

    stream
        .flush()
        .await
        .map_err(|e| ConnectionError::SendReadyError(e))?;

    let mut download_successful = false;

    for current_retry in 0..FILE_RETRIES {
        match handle_file::handle_file(&server_config, &backup_config, stream).await {
            Ok(_) => {
                if current_retry != 0 {
                    info!("Attempt {} - Successfully downloaded", current_retry);
                }
                download_successful = true;
            }
            Err(error) => match error {
                handle_file::HandleFileError::HashError => {
                    warn!(
                        "Attempt {} - Failed to download file: {}",
                        current_retry, error
                    );
                    stream
                        .write_all(b"retry")
                        .await
                        .map_err(|e| ConnectionError::SendRetryError(e))?;
                    stream
                        .flush()
                        .await
                        .map_err(|e| ConnectionError::SendRetryError(e))?;
                }
                _ => return Err(ConnectionError::DownloadError(error)),
            },
        };
    }

    if !download_successful {
        return Err(ConnectionError::RetryError);
    }

    info!("Download successful");

    Ok(())
}
