pub mod accept;
pub mod cleanup;
pub mod handle;

use std::{net::TcpListener, sync::Arc};

use log::{error, info};
use rustls::server::{NoServerSessionStorage, VerifierBuilderError, WebPkiClientVerifier};
use shared::{CertificateError, Certificates};
use thiserror::Error;

use crate::{
    ip_list::{self, IpList},
    server_config::{self, ServerConfig},
};

/// The backups server.
pub struct Server {
    #[allow(unused)]
    config: ServerConfig,
    ip_list: IpList,
    tls_config: Arc<rustls::ServerConfig>,
    listener: TcpListener,
}

impl Server {
    /// Creates a new server.
    pub fn new() -> Result<Self, Error> {
        // Load config from files
        let config = ServerConfig::try_load()?;
        let ip_list = IpList::load_or_create().map_err(Error::LoadIpList)?;
        let certificates = Certificates::load(
            &config.root_ca_path,
            &config.certificate_path,
            &config.key_path,
        )?;

        // Setup TLS listener
        let client_cert_verifier =
            WebPkiClientVerifier::builder(Arc::new(certificates.root_cert_store)).build()?;

        let mut tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(certificates.certificates, certificates.key)
            .map_err(Error::TlsConfig)?;

        tls_config.session_storage = Arc::new(NoServerSessionStorage {});

        let tls_config = Arc::new(tls_config);
        let listener = TcpListener::bind(config.socket_address).map_err(Error::Bind)?;

        info!("Bound on {}", config.socket_address);

        Ok(Self {
            config,
            ip_list,
            tls_config,
            listener,
        })
    }

    #[cfg(test)]
    /// Creates a new test server
    pub fn new_test(
        pki: &crate::tests::TestPki,
        address: &std::net::SocketAddr,
    ) -> Result<Self, Error> {
        use rustls_pki_types::PrivatePkcs8KeyDer;

        // Setup TLS listener
        let client_cert_verifier = WebPkiClientVerifier::builder(pki.roots.clone()).build()?;

        let mut tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(
                vec![pki.server_cert.cert.der().clone()],
                PrivatePkcs8KeyDer::from(pki.server_cert.key_pair.serialize_der()).into(),
            )
            .map_err(Error::TlsConfig)?;

        tls_config.session_storage = Arc::new(NoServerSessionStorage {});

        let tls_config = Arc::new(tls_config);
        let listener = TcpListener::bind(address).map_err(Error::Bind)?;

        info!("Bound on {}", address);

        Ok(Self {
            config: ServerConfig::blank(),
            ip_list: IpList::new_unbacked(),
            tls_config,
            listener,
        })
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to load server config:\n{0}")]
    LoadConfig(#[from] server_config::Error),

    #[error("Failed to load or create ip_list:\n{0}")]
    LoadIpList(#[source] ip_list::Error),

    #[error("Failed to load certificates:\n{0}")]
    LoadCertificates(#[from] CertificateError),

    #[error("Failed to create client verifier\n{0}")]
    CreateVerifier(#[from] VerifierBuilderError),

    #[error("Failed to create TLS config:\n{0}")]
    TlsConfig(#[source] rustls::Error),

    #[error("Failed to bind TCP listener:\n{0}")]
    Bind(#[source] std::io::Error),
}
