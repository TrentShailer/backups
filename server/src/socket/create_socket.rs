use std::{io, sync::Arc};

use log::info;
use rustls::server::{VerifierBuilderError, WebPkiClientVerifier};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::config::TlsConfig;

pub async fn create_socket(
    config: TlsConfig,
) -> Result<(TcpListener, TlsAcceptor), CreateSocketError> {
    let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(config.root_ca)).build()?;

    let tls_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(config.certificate_chain, config.private_key)?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind(&config.socket_address).await?;

    info!("Listening on address: {0}", config.socket_address);

    Ok((listener, tls_acceptor))
}

#[derive(Debug, Error)]
pub enum CreateSocketError {
    #[error("BuildClientVerifierError[br]{0}")]
    BuildClientVerifierError(#[from] VerifierBuilderError),
    #[error("BuildTlsConfigError[br]{0}")]
    BuildTlsConfigError(#[from] rustls::Error),
    #[error("TcpBindError[br]{0}")]
    TcpBindError(#[from] io::Error),
}
