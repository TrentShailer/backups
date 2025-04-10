//! # common
//!

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
    net::TcpListener,
    path::PathBuf,
    sync::Arc,
};

use backup_receiver::{Config, Receiver};
use rustls::server::{NoServerSessionStorage, WebPkiClientVerifier};
use shared::Metadata;

pub fn test_receiver() -> Receiver {
    let config = Config::default();

    // Setup TLS config
    let tls_config = {
        let (certificate_authority, root_cert_store) = shared::test::new_certificate_authority();
        let (key, certificate) = shared::test::new_signed(&certificate_authority);
        let private_key = shared::test::private_key_der(&key);

        let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(root_cert_store))
            .build()
            .unwrap();

        let mut tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(vec![certificate.der().clone()], private_key)
            .unwrap();

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
        fs::remove_dir_all(&dir).unwrap(); // TODO this makes me feel uneasy
        fs::remove_dir(dir.parent().unwrap()).unwrap(); // TODO this makes me feel uneasy
    } else {
        fs::remove_file(&dir).unwrap();
    }
}

pub fn check_backup(metadata: &Metadata, payload: &[u8]) {
    let dir = metadata.backup_directory();

    let dir_metadata =
        fs::metadata(&dir).unwrap_or_else(|e| panic!("Backup dir should exist: {dir:?}: {e}"));

    if !dir_metadata.is_dir() {
        panic!("Backup directory should be a directory: {dir:?}");
    }

    let directory: Vec<_> = fs::read_dir(dir).unwrap().collect();
    assert_eq!(directory.len(), 1);

    for file in directory {
        let file = file.unwrap();
        let contents = fs::read_to_string(file.path()).unwrap();
        assert_eq!(contents.as_bytes(), payload);
    }
}
