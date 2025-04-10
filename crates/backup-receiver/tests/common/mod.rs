//! # common
//!

use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{collections::HashMap, io::ErrorKind, net::TcpListener, sync::Arc};

use backup_receiver::{Config, Receiver};
use rustls::server::{NoServerSessionStorage, WebPkiClientVerifier};

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
