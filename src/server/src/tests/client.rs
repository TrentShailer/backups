use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    sync::Arc,
};

use rustls::{ClientConfig, ClientConnection, Stream};
use rustls_pki_types::{PrivatePkcs8KeyDer, ServerName};
use shared::BackupMetadata;
use thiserror::Error;

use super::TestPki;

pub struct TestClient {
    tls_config: Arc<ClientConfig>,
    address: SocketAddr,
}

impl TestClient {
    pub fn new(pki: &TestPki, address: SocketAddr) -> Self {
        let tls_config = ClientConfig::builder()
            .with_root_certificates(pki.roots.clone())
            .with_client_auth_cert(
                vec![pki.client_cert.cert.der().clone()],
                PrivatePkcs8KeyDer::from(pki.client_cert.key_pair.serialize_der()).into(),
            )
            .unwrap();

        Self {
            tls_config: Arc::new(tls_config),
            address,
        }
    }

    pub fn try_make_backup(&self, file_name: &'static str) -> Result<(), Error> {
        // Connect via TCP
        let mut socket = TcpStream::connect(self.address).map_err(Error::Tcp)?;

        // Connect via TLS
        let server_name: ServerName = "127.0.0.1".try_into().expect("Invalid DNS name");
        let mut client = ClientConnection::new(self.tls_config.clone(), server_name)
            .map_err(Error::ClientConnection)?;

        // Complete handshake with server to ensure authentication
        client
            .complete_io(&mut socket)
            .map_err(Error::CompleteHandshake)?;

        let mut stream = Stream::new(&mut client, &mut socket);

        let metadata = BackupMetadata {
            backup_size: 0,
            file_name: file_name.to_string(),
            service_name: "backups_server".to_string(),
            backup_name: "tests".to_string(),
            max_files: 2,
        };
        let metadata_string = toml::to_string(&metadata).expect("Failed to parse metadata string");

        // Write metadata hint then metadata
        stream
            .write_all(&metadata_string.len().to_be_bytes())
            .map_err(|e| Error::Write(e, "metadata hint"))?;

        stream
            .write_all(metadata_string.as_bytes())
            .map_err(|e| Error::Write(e, "metadata"))?;

        /* Writing would normally be done here */

        // Flush the stream
        stream.flush().map_err(Error::Flush)?;

        // Wait for OK
        let mut result_buffer = [0u8; 1];
        stream
            .read_exact(&mut result_buffer)
            .map_err(|e| Error::Read(e, "result response"))?;

        // Complete IO
        stream.conn.send_close_notify();
        stream
            .conn
            .complete_io(stream.sock)
            .map_err(Error::CompleteIo)?;

        if result_buffer[0] == 0 {
            return Err(Error::FailureResponse);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Tcp Error:\n{0}")]
    Tcp(#[source] io::Error),

    #[error("CompleteHandshake Error:\n{0}")]
    CompleteHandshake(#[source] io::Error),

    #[error("Tcp Error:\n{0}")]
    ClientConnection(#[source] rustls::Error),

    #[error("Failed to write {1}:\n{0}")]
    Write(#[source] io::Error, &'static str),

    #[error("Failed to read {1}:\n{0}")]
    Read(#[source] io::Error, &'static str),

    #[error("Failed to flush stream:\n{0}")]
    Flush(#[source] io::Error),

    #[error("Failed to complete IO:\n{0}")]
    CompleteIo(#[source] io::Error),

    #[error("Got failure response")]
    FailureResponse,
}
