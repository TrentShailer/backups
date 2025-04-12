use core::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use std::{
    collections::HashMap,
    io::{self, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    time::Instant,
};

use rustls::{
    ServerConnection, Stream,
    server::{Acceptor, NoServerSessionStorage, VerifierBuilderError, WebPkiClientVerifier},
};
use shared::{CertificateError, Certificates, Response};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::{Config, cleanup, context::Context};

mod handle_client;

/// The backup receiver.
pub struct Receiver {
    /// The receiver config,
    pub config: Config,

    /// The TLS config.
    pub tls_config: Arc<rustls::ServerConfig>,

    /// The TCP listener.
    pub listener: TcpListener,

    /// The last 60 minutes of backups per IP address.
    pub history: HashMap<IpAddr, Vec<Instant>>,
}

impl Receiver {
    /// Create a new receiver from config.
    pub fn new(config: Config) -> Result<Self, CreateReceiverError> {
        // TODO IpList?

        // Setup TLS config
        let tls_config = {
            let certificates = Certificates::load(
                &config.tls.root_certificate_file,
                &config.tls.certificate_file,
                &config.tls.private_key_file,
            )?;

            let client_cert_verifier =
                WebPkiClientVerifier::builder(Arc::new(certificates.trust_store)).build()?;

            let mut tls_config = rustls::ServerConfig::builder()
                .with_client_cert_verifier(client_cert_verifier)
                .with_single_cert(certificates.certificate_chain, certificates.private_key)
                .map_err(CreateReceiverError::TlsConfig)?;

            tls_config.session_storage = Arc::new(NoServerSessionStorage {});

            Arc::new(tls_config)
        };

        // Bind TCP listener
        let listener =
            TcpListener::bind(config.socket_address).map_err(CreateReceiverError::Bind)?;

        Ok(Self {
            config,
            tls_config,
            listener,
            history: HashMap::default(),
        })
    }

    /// Accept and handle a client.
    pub fn accept_and_handle_client(&mut self) {
        let mut context = Context::default();

        let (mut connection, mut stream, peer) = match self.accept_client(&mut context) {
            Ok(client) => client,
            Err(error) => {
                warn!("{context}Failed to accept mTLS connection: {error}");
                return;
            }
        };

        let mut stream = Stream::new(&mut connection, &mut stream);

        let metadata = match self.handle_client(&mut context, &mut stream, peer) {
            Ok(metadata) => {
                self.send_response_and_close(&mut context, &mut stream, Response::Success);
                metadata
            }
            Err(response) => {
                self.send_response_and_close(&mut context, &mut stream, response);
                return;
            }
        };

        // Track backup in history
        if let Some(history) = self.history.get_mut(&peer.ip()) {
            history.push(Instant::now());
        } else {
            self.history.insert(peer.ip(), vec![Instant::now()]);
        }

        // Clean up files
        cleanup(&mut context, &self.config, &metadata);
    }

    /// Block until a client connects then accept the mTLS connection.
    pub fn accept_client(
        &mut self,
        context: &mut Context,
    ) -> Result<(ServerConnection, TcpStream, SocketAddr), AcceptError> {
        context.current_context = "Accept Client";

        // Accept TCP connection
        let (mut stream, peer) = self.listener.accept().map_err(AcceptError::AcceptTcp)?;

        // Set timeouts
        {
            stream
                .set_read_timeout(Some(Duration::from_secs(
                    self.config.limits.timeout_seconds,
                )))
                .expect("Timeout must not be zero");
            stream
                .set_write_timeout(Some(Duration::from_secs(
                    self.config.limits.timeout_seconds,
                )))
                .expect("Timeout must not be zero");
        }

        context.peer = Some(peer.ip());
        info!("{context}Connected");

        // Try accept TLS connection
        let accepted = {
            // Read Client Hello
            let mut acceptor = Acceptor::default();
            loop {
                acceptor
                    .read_tls(&mut stream)
                    .map_err(AcceptError::ReadTls)?;

                match acceptor.accept() {
                    Ok(Some(accepted)) => break accepted,
                    Ok(None) => continue,
                    Err((e, mut alert)) => {
                        if let Err(e) = alert.write_all(&mut stream) {
                            warn!("{context}Could not write TLS accept failed alert: {e}");
                        }

                        return Err(AcceptError::AcceptTls(e));
                    }
                };
            }
        };

        // Try get a connection
        let mut connection = accepted
            .into_connection(Arc::clone(&self.tls_config))
            .map_err(|(e, mut alert)| {
                if let Err(e) = alert.write_all(&mut stream) {
                    warn!("{context}Could not write TLS accept failed alert: {e}");
                }

                AcceptError::CreateConnection(e)
            })?;

        // Complete handshake
        connection
            .complete_io(&mut stream)
            .map_err(AcceptError::CompleteIo)?;

        info!("{context}Accepted {:?}", connection.protocol_version());

        Ok((connection, stream, peer))
    }

    /// Send a response to the sender and close the connection.
    pub fn send_response_and_close(
        &self,
        context: &mut Context,
        stream: &mut Stream<'_, ServerConnection, TcpStream>,
        response: Response,
    ) {
        context.current_context = "Send Response";

        if response != Response::Success {
            warn!("{context}Sending {response:?}")
        }

        let response_bytes = response.to_be_bytes();
        if let Err(error) = stream.write_all(&response_bytes) {
            error!("{context}Could not write response: {error}");
        };

        stream.conn.send_close_notify();
        if let Err(error) = stream.conn.complete_io(stream.sock) {
            error!("{context}Could not complete io: {error}");
        };
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum AcceptError {
    #[error("Failed to accept TCP connection: {0}")]
    AcceptTcp(#[source] io::Error),

    #[error("Failed to read TLS: {0}")]
    ReadTls(#[source] io::Error),

    #[error("Failed to accept TLS: {0}")]
    AcceptTls(#[source] rustls::Error),

    #[error("Failed to create connection: {0}")]
    CreateConnection(#[source] rustls::Error),

    #[error("Failed to complete io: {0}")]
    CompleteIo(#[source] io::Error),
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CreateReceiverError {
    #[error("Failed to load certificates:\n{0}")]
    LoadCertificates(#[from] CertificateError),

    #[error("Failed to create client verifier:\n{0}")]
    ClientVerifier(#[from] VerifierBuilderError),

    #[error("Failed to create TLS server config:\n{0}")]
    TlsConfig(#[source] rustls::Error),

    #[error("Failed to bind TCP listener:\n{0}")]
    Bind(#[source] io::Error),
}
