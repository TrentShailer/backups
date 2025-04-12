//! # common
//!

#![allow(unused)]

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{
    collections::HashMap,
    fs::{self, ReadDir},
    io::ErrorKind,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use backup_receiver::{Config, Receiver};
use rcgen::{Certificate, KeyPair};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, Stream,
    pki_types::ServerName,
    server::{NoServerSessionStorage, WebPkiClientVerifier},
};
use shared::{
    Metadata,
    test::{CertificateAuthority, private_key_der},
};

pub fn test_receiver(certificate_authority: &CertificateAuthority) -> Receiver {
    let config = Config::default();

    // Setup TLS config
    let tls_config = {
        let (key, certificate) = certificate_authority.generate_signed();
        let private_key = private_key_der(&key);

        let client_cert_verifier =
            WebPkiClientVerifier::builder(Arc::new(certificate_authority.certificate_store()))
                .build()
                .unwrap();

        let mut tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(vec![certificate.der().clone()], private_key)
            .unwrap();

        tls_config.send_tls13_tickets = 0;
        tls_config.session_storage = Arc::new(NoServerSessionStorage {});

        Arc::new(tls_config)
    };

    // Bind TCP listener
    let listener = {
        let mut port = 8081;
        let mut address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let mut listener = TcpListener::bind(address);
        while listener
            .as_ref()
            .is_err_and(|e| e.kind() == ErrorKind::AddrInUse)
        {
            port += 1;
            address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            listener = TcpListener::bind(address);
        }

        listener.unwrap()
    };

    Receiver {
        config,
        tls_config,
        listener,
        history: HashMap::default(),
    }
}

pub fn test_client(
    key: KeyPair,
    certificate: Certificate,
    roots: RootCertStore,
    address: SocketAddr,
) -> (TcpStream, ClientConnection) {
    let tls_config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(vec![certificate.der().clone()], private_key_der(&key))
        .unwrap();

    // Connect via TCP
    let mut socket = TcpStream::connect(address).unwrap();

    // Connect via TLS
    let server_name: ServerName<'_> = "127.0.0.1".try_into().expect("Invalid DNS name");
    let mut client = ClientConnection::new(tls_config.into(), server_name).unwrap();

    // Complete handshake with server to ensure authentication
    client.complete_io(&mut socket).unwrap();

    (socket, client)
}

pub fn clear_backups(metadata: &Metadata) {
    let dir = metadata.backup_directory();

    let metadata = match fs::metadata(&dir) {
        Ok(metadata) => metadata,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return;
            } else {
                panic!("{}", e);
            }
        }
    };

    if metadata.is_dir() {
        // delete all files
        fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    } else {
        fs::remove_file(&dir).unwrap();
    }
}

pub fn backup_dir(metadata: &Metadata) -> ReadDir {
    let dir = metadata.backup_directory();

    let dir_metadata =
        fs::metadata(&dir).unwrap_or_else(|e| panic!("Backup dir should exist: {dir:?}: {e}"));

    if !dir_metadata.is_dir() {
        panic!("Backup directory should be a directory: {dir:?}");
    }

    fs::read_dir(dir).unwrap()
}

pub fn check_backup_payload(metadata: &Metadata, payload: &[u8]) {
    let directory: Vec<_> = backup_dir(metadata).collect();
    assert_eq!(directory.len(), 1);

    for file in directory {
        let file = file.unwrap();
        let contents = fs::read_to_string(file.path()).unwrap();
        assert_eq!(contents.as_bytes(), payload);
    }
}
