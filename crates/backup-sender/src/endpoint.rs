//! Endpoint to receive the backup.
//!

use core::num::TryFromIntError;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    path::PathBuf,
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, Stream, pki_types::ServerName};
use serde::{Deserialize, Serialize};
use shared::{Certificates, Failure, Response};
use thiserror::Error;

/// Endpoint for a backup receiver.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Endpoint {
    /// The address of the backup receiver.
    pub receiver_address: String,

    /// The port of the backup receiver.
    pub receiver_port: u16,

    /// The path to the sender certificate.
    pub certificate_file: PathBuf,

    /// The path to the sender key.
    pub private_key_file: PathBuf,

    /// The path to the trusted root certificate.
    pub root_certificate_file: PathBuf,
}

impl Endpoint {
    /// Send a backup to the endpoint.
    pub fn send_backup(&self, mut backup: crate::Backup) -> Result<(), SendBackupError> {
        // Load certificates and setup TLS config
        let certificates = Certificates::load(
            &self.root_certificate_file,
            &self.certificate_file,
            &self.private_key_file,
        )
        .or_log_and_panic("Failed to load certificates");

        let tls_config = ClientConfig::builder()
            .with_root_certificates(certificates.trust_store)
            .with_client_auth_cert(certificates.certificate_chain, certificates.private_key)
            .or_log_and_panic("Certificates are invalid");

        // Connect via TCP
        let mut socket = TcpStream::connect((self.receiver_address.clone(), self.receiver_port))
            .map_err(SendBackupError::TcpConnect)?;

        // Connect via TLS
        let server_name: ServerName<'_> = ServerName::try_from(self.receiver_address.clone())
            .or_log_and_panic("Invalid receiver address");
        let mut client = ClientConnection::new(Arc::new(tls_config), server_name)
            .map_err(SendBackupError::TlsConnect)?;

        // Complete handshake with server to ensure authentication
        client
            .complete_io(&mut socket)
            .map_err(|e| SendBackupError::Io(e, "complete handshake"))?;
        let mut stream = Stream::new(&mut client, &mut socket);

        // Write the metadata
        stream
            .write_all(&backup.metadata.as_be_bytes())
            .map_err(|e| SendBackupError::Io(e, "write metadata"))?;

        // Write the payload
        {
            let mut read_buffer = [0u8; 1024];
            let mut total_bytes_read = 0;
            let backup_size = usize::try_from(backup.metadata.backup_bytes)?;

            while total_bytes_read < backup_size {
                let bytes_read = backup
                    .reader
                    .read(&mut read_buffer)
                    .map_err(|e| SendBackupError::Io(e, "read payload"))?;

                stream
                    .write_all(&read_buffer[..bytes_read])
                    .map_err(|e| SendBackupError::Io(e, "write payload"))?;

                total_bytes_read += bytes_read;
            }
        }

        // Flush stream
        stream
            .flush()
            .map_err(|e| SendBackupError::Io(e, "flush"))?;

        // Read response
        let response: Response = {
            let mut response_buffer = [0u8; size_of::<Response>()];
            stream
                .read_exact(&mut response_buffer)
                .map_err(|e| SendBackupError::Io(e, "read response"))?;

            let value = u64::from_be_bytes(response_buffer);

            match Response::try_from_u64(value) {
                Some(response) => response,
                None => return Err(SendBackupError::InvalidResponse),
            }
        };

        // Complete IO
        stream.conn.send_close_notify();
        stream
            .conn
            .complete_io(stream.sock)
            .map_err(|e| SendBackupError::Io(e, "complete IO"))?;

        if response == Response::Success {
            Ok(())
        } else {
            Err(SendBackupError::ErrorResponse(response))
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum SendBackupError {
    #[error("Failed to make TCP connection: {0}")]
    TcpConnect(#[source] io::Error),

    #[error("Failed to make TLS connection: {0}")]
    TlsConnect(#[source] rustls::Error),

    #[error("Failed to {1}: {0}")]
    Io(#[source] io::Error, &'static str),

    #[error("Backup size exceeded usize::MAX: {0}")]
    BackupTooLarge(#[from] TryFromIntError),

    #[error("Response was an invalid value")]
    InvalidResponse,

    #[error("Response was an error: {0:?}")]
    ErrorResponse(Response),
}
