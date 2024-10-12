use std::{
    io::{self, Write},
    net::TcpStream,
};

use log::{error, info};
use rustls::{
    server::{Accepted, Acceptor},
    Stream,
};
use thiserror::Error;

use crate::ip_list;

use super::{handle, Server};

impl Server {
    /// Blocks until a connection, then accepts and handles it.
    pub fn accept_blocking(&mut self) -> Result<(), Error> {
        // Accept TCP connection
        let (mut stream, peer) = self.listener.accept().map_err(Error::Accept)?;

        log::info!("Peer connected: {}", peer);

        // Check blocklist
        if self.ip_list.is_blocked(&peer.ip()) {
            return Ok(());
        }

        // Try accept TLS connection, try block the IP if hello fails.
        let accepted = match Self::try_accept_hello(&mut stream) {
            Ok(accepted) => accepted,
            Err(e) => {
                self.ip_list.block_untrusted(peer.ip())?;
                return Err(e);
            }
        };

        // Try get a connection from the acceptor, try block the IP if this fails.
        let mut connection = match accepted.into_connection(self.tls_config.clone()) {
            Ok(connection) => connection,
            Err((e, mut alert)) => {
                alert.write_all(&mut stream).map_err(Error::WriteAlert)?;
                self.ip_list.block_untrusted(peer.ip())?;
                return Err(Error::AcceptTls(e));
            }
        };

        // Complete handshake with client to ensure authentication
        if let Err(e) = connection.complete_io(&mut stream) {
            self.ip_list.block_untrusted(peer.ip())?;
            return Err(Error::CompleteHandshake(e));
        };

        let mut stream = Stream::new(&mut connection, &mut stream);

        info!("Peer accepted: {}", peer);

        let result = Self::handle_connection(&mut stream);

        // Send result response
        let result_response = if result.is_ok() { 1 } else { 0 };
        if let Err(e) = stream.write_all(&[result_response]) {
            error!("Failed to write result response:\n{e}");
        };

        // Complete io
        stream.conn.send_close_notify();
        if let Err(e) = stream.conn.complete_io(stream.sock) {
            error!("Failed to complete io:\n{e}");
        };

        result?;

        // This client sent a payload successfully, they are trusted and shouldn't get blocked
        // if they error in the future.
        self.ip_list.trust_unblocked(peer.ip())?;

        Ok(())
    }

    /// Tries to accept a TLS hello from the stream.
    fn try_accept_hello(stream: &mut TcpStream) -> Result<Accepted, Error> {
        let mut acceptor = Acceptor::default();
        let accepted = loop {
            acceptor.read_tls(stream).map_err(Error::ReadHelloTls)?;

            match acceptor.accept() {
                Ok(Some(accepted)) => break accepted,
                Ok(None) => continue,
                Err((e, mut alert)) => {
                    alert.write_all(stream).map_err(Error::WriteAlert)?;
                    return Err(Error::AcceptTls(e));
                }
            }
        };

        Ok(accepted)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to accept connection:\n{0}")]
    Accept(#[source] io::Error),

    #[error("Failed to update ip list:\n{0}")]
    IpList(#[from] ip_list::Error),

    #[error("Failed to read TLS hello:\n{0}")]
    ReadHelloTls(#[source] io::Error),

    #[error("Failed to complete handshake:\n{0}")]
    CompleteHandshake(#[source] io::Error),

    #[error("Failed to write TLS hello alert:\n{0}")]
    WriteAlert(#[source] io::Error),

    #[error("Failed to accept TLS connection:\n{0}")]
    AcceptTls(#[source] rustls::Error),

    #[error("Failed to handle connection:\n{0}")]
    Handle(#[from] handle::Error),
}
