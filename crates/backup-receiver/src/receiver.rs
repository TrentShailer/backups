use core::net::{IpAddr, SocketAddr};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    time::Instant,
};

use bytemuck::{bytes_of, checked};
use chrono::Utc;
use rustls::{
    ServerConnection, Stream,
    server::{Acceptor, NoServerSessionStorage, VerifierBuilderError, WebPkiClientVerifier},
};
use shared::{CertificateError, Certificates, Metadata, Response};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::{Config, context_logger::ContextLogger};

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

    /// Block until a client connects then accept the mTLS connection.
    pub fn accept_blocking(
        &mut self,
        context: &mut ContextLogger,
    ) -> Result<(ServerConnection, TcpStream, SocketAddr), AcceptError> {
        context.current_context = "Accept";

        // Accept TCP connection
        let (mut stream, peer) = self.listener.accept().map_err(AcceptError::AcceptTcp)?;
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

        info!("{context}Accepted");

        Ok((connection, stream, peer))
    }

    /// Read the metadata from the stream.
    pub fn read_metadata(
        &self,
        context: &mut ContextLogger,
        stream: &mut Stream<'_, ServerConnection, TcpStream>,
    ) -> Result<Metadata, Response> {
        context.current_context = "Read Metadata";

        let mut buffer = [0u8; size_of::<Metadata>()];

        // Read the exact expected bytes for metadata.
        stream.read_exact(&mut buffer).map_err(|e| {
            if e.kind() == ErrorKind::UnexpectedEof {
                warn!("{context}Encountered unexpected Eof when reading metadata: {e}");
                Response::BadData
            } else {
                error!("{context}Encountered error when reading metadata: {e}");
                Response::Error
            }
        })?;

        // Try cast the bytes to a Metadata instance.
        let metadata: Metadata = *checked::try_from_bytes(&buffer).map_err(|e| {
            warn!("{context}Invalid metadata: {e}");
            Response::BadData
        })?;

        context.backup = Some((metadata.serivce_name().to_string(), metadata.cadance));

        info!("{context}Received metadata");

        // Check limits
        if metadata.backup_bytes > self.config.limits.maximum_payload_bytes {
            warn!(
                "{context}Exceeded payload size limit {} > {}",
                metadata.backup_bytes, self.config.limits.maximum_payload_bytes
            );
            return Err(Response::TooLarge);
        }

        Ok(metadata)
    }

    /// Prepare the backup directory and open the backup file.
    pub fn prepare_backup_file(
        &self,
        context: &mut ContextLogger,
        metadata: &Metadata,
    ) -> Result<File, Response> {
        context.current_context = "Prepare Backup";

        let backup_directory = metadata.backup_directory();

        // Create backup directory if it doesn't exist

        // Check if the backup dir exists
        let directroy_metadata = match fs::metadata(&backup_directory) {
            Ok(dir_metadata) => Some(dir_metadata),
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    None
                } else {
                    error!("{context}Could not check metadata for {backup_directory:?}: {error}");
                    return Err(Response::Error);
                }
            }
        };

        match directroy_metadata {
            // If the backup_dir exists, ensure it is a directory
            Some(directory_metadata) => {
                if !directory_metadata.is_dir() {
                    error!("{context}{backup_directory:?} is not a dir: {directory_metadata:?}");
                    return Err(Response::Error);
                }
            }

            // If it does not exist, create it.
            None => {
                fs::create_dir_all(&backup_directory).map_err(|e| {
                    error!("{context}Could not create directory {backup_directory:?}: {e}");
                    Response::Error
                })?;
            }
        }

        let file_name = format!(
            "{}.{}",
            Utc::now().format("%Y-%m-%d_%H-%M-%S"),
            metadata.file_exteion()
        );
        let backup_file_path = backup_directory.join(file_name);

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&backup_file_path)
            .map_err(|e| {
                error!("{context}Could not create and open file at {backup_file_path:?}: {e}");
                Response::Error
            })?;

        Ok(file)
    }

    /// Read the payload from the stream and write it to the file.
    pub fn read_and_write_payload(
        &self,
        context: &mut ContextLogger,
        stream: &mut Stream<'_, ServerConnection, TcpStream>,
        metadata: &Metadata,
        file: &mut File,
    ) -> Result<(), Response> {
        context.current_context = "Read Write Payload";

        // Setup 1 KiB buffer for reading
        let mut file_buffer = [0u8; 1024];
        let mut total_bytes_read: usize = 0;

        let backup_bytes = metadata.backup_bytes;
        let backup_bytes = usize::try_from(backup_bytes).map_err(|e| {
            error!("{context}Could not cast backup bytes {backup_bytes} to usize: {e}");
            Response::Error
        })?;

        // Read the payload in chunks and append the chunks to the output file.
        while total_bytes_read < backup_bytes {
            let bytes_read = stream.read(&mut file_buffer[..]).map_err(|e| {
                if e.kind() == ErrorKind::UnexpectedEof {
                    warn!("{context}Encountered unexpected Eof when reading payload: {e}");
                    Response::BadData
                } else {
                    error!("{context}Encountered error when reading payload: {e}");
                    Response::Error
                }
            })?;

            file.write_all(&file_buffer[..bytes_read]).map_err(|e| {
                error!("{context}Encountered error when writing to backup file: {e}");
                Response::Error
            })?;

            total_bytes_read += bytes_read;
        }

        Ok(())
    }

    /// Send a response to the sender and close the connection.
    pub fn send_response_and_close(
        &self,
        context: &mut ContextLogger,
        stream: &mut Stream<'_, ServerConnection, TcpStream>,
        response: Response,
    ) {
        context.current_context = "Send Response";

        if response != Response::Success {
            warn!("{context}Sending {response:?}")
        }

        let response_bytes = bytes_of(&response);
        if let Err(error) = stream.write_all(response_bytes) {
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
