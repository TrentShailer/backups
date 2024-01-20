use std::{
    fs::File,
    io::{self, BufReader},
};

use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

use crate::server_config::ServerConfig;

pub struct Certificates {
    pub certificates: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub root_cert_store: RootCertStore,
}

pub fn load_certificates(config: &ServerConfig) -> Result<Certificates, Error> {
    // Cert
    let cert_file = File::open(&config.certificate_path)?;
    let certificates: Vec<CertificateDer<'_>> =
        certs(&mut BufReader::new(cert_file)).collect::<io::Result<_>>()?;

    // Key
    let key_file = File::open(&config.key_path)?;
    let key = private_key(&mut BufReader::new(key_file))?.unwrap();

    // Root CA
    let root_ca_file = File::open(&config.root_ca_path)?;

    let root_certs: Vec<CertificateDer<'_>> =
        certs(&mut BufReader::new(root_ca_file)).collect::<io::Result<_>>()?;

    let mut root_cert_store = RootCertStore::empty();
    for cert in root_certs {
        root_cert_store.add(cert.clone())?;
    }

    Ok(Certificates {
        certificates,
        key,
        root_cert_store,
    })
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IoError:\n{0}")]
    Io(#[from] io::Error),
    #[error("RootStoreError:\n{0}")]
    RootStore(#[from] rustls::Error),
}
