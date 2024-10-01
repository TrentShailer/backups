use std::{
    io::{self, Read, Write},
    net::TcpStream,
    path::PathBuf,
    sync::Arc,
};

use chrono::Local;
use log::error;
use rustls::{ClientConfig, ClientConnection, Stream};
use rustls_pki_types::{InvalidDnsNameError, ServerName};
use serde::{Deserialize, Serialize};
use shared::{BackupMetadata, CertificateError, Certificates};
use thiserror::Error;

use crate::service::BackupContents;

/// Endpoint that is a TlsServer that follows the implementation in `backups-server`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsServer {
    pub socket_address: String,
    pub socket_port: u16,
    pub certificate_path: PathBuf,
    pub key_path: PathBuf,
    pub root_ca_path: PathBuf,
}

impl TlsServer {
    /// Backup data to the TLS endpoint
    pub fn make_backup(
        &self,
        service_name: String,
        backup_name: String,
        max_files: u64,
        backup_contents: &mut BackupContents,
    ) -> Result<(), Error> {
        // Enforce 64-bit usize to make conversions between u64 and usize safe
        if usize::BITS != 64 {
            panic!("usize is not 64-bits");
        }

        // Construct metadata
        let metadata = BackupMetadata {
            backup_size: backup_contents.backup_size,
            file_name: Local::now().format("%Y-%m-%d_%H-%M-%S.backup").to_string(),
            backup_name,
            service_name,
            max_files,
        };
        let metadata_string = toml::to_string(&metadata)?;

        // Load certificates and setup TLS config
        let certs = Certificates::load(&self.root_ca_path, &self.certificate_path, &self.key_path)?;
        let tls_config = ClientConfig::builder()
            .with_root_certificates(certs.root_cert_store)
            .with_client_auth_cert(certs.certificates, certs.key)
            .map_err(Error::TlsConfig)?;

        // Connect via TCP
        let mut socket = TcpStream::connect((self.socket_address.clone(), self.socket_port))
            .map_err(Error::ConnectTcp)?;

        // Connect via TLS
        let server_name: Result<ServerName, _> = self.socket_address.clone().try_into();
        let server_name = server_name?;
        let mut client =
            ClientConnection::new(Arc::new(tls_config), server_name).map_err(Error::Connection)?;

        // Complete handshake with server to ensure authentication
        client
            .complete_io(&mut socket)
            .map_err(Error::CompleteHandshake)?;

        let mut stream = Stream::new(&mut client, &mut socket);

        // Write metadata hint then metadata
        let metadata_len = metadata_string.len() as u64;
        stream
            .write_all(&metadata_len.to_be_bytes())
            .map_err(|e| Error::Write(e, "metadata hint"))?;

        stream
            .write_all(metadata_string.as_bytes())
            .map_err(|e| Error::Write(e, "metadata"))?;

        // Read from reader in 1KiB chunks and write the chunks to the TLS stream.
        let mut read_buffer = [0u8; 1024];
        let mut total_bytes_read = 0;
        let backup_size = backup_contents.backup_size as usize;

        while total_bytes_read < backup_size {
            let bytes_read = backup_contents
                .reader
                .read(&mut read_buffer)
                .map_err(|e| Error::Read(e, "from reader"))?;

            stream
                .write_all(&read_buffer[..bytes_read])
                .map_err(|e| Error::Write(e, "payload chunk"))?;

            total_bytes_read += bytes_read;
        }

        // Flush the stream
        stream.flush().map_err(Error::Flush)?;

        // Wait for OK
        let mut result_buffer = [0u8; 1];
        if let Err(e) = stream.read_exact(&mut result_buffer) {
            error!("Failed to read result response:\n{e}");
        };

        // Complete IO
        stream.conn.send_close_notify();
        if let Err(e) = stream.conn.complete_io(stream.sock) {
            error!("Failed to read result response:\n{e}");
        }

        if result_buffer[0] == 0 {
            return Err(Error::FailureResponse);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to serialize metadata:\n{0}")]
    SerializeMetadata(#[from] toml::ser::Error),

    #[error("Failed to load certificates:\n{0}")]
    LoadCertificates(#[from] CertificateError),

    #[error("Failed to create TLS config:\n{0}")]
    TlsConfig(#[source] rustls::Error),

    #[error("Failed to make TCP connection:\n{0}")]
    ConnectTcp(#[source] io::Error),

    #[error("Failed to complete handshake:\n{0}")]
    CompleteHandshake(#[source] io::Error),

    #[error("Failed to make server name:\n{0}")]
    InvalidDnsName(#[from] InvalidDnsNameError),

    #[error("Failed to make TLS connection:\n{0}")]
    Connection(#[source] rustls::Error),

    #[error("Failed to write {1}:\n{0}")]
    Write(#[source] io::Error, &'static str),

    #[error("Failed to read {1}:\n{0}")]
    Read(#[source] io::Error, &'static str),

    #[error("Failed to flush stream:\n{0}")]
    Flush(#[source] io::Error),

    #[error("Got failure response")]
    FailureResponse,
}
