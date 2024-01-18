use std::{
    fs::File,
    io::{self, BufReader},
};

use futures_rustls::rustls::RootCertStore;
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
    let cert_file = File::open(&config.certificate_path).map_err(|e| Error::ReadFile("cert", e))?;

    let certificates = certs(&mut BufReader::new(cert_file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(|e| Error::LoadCert("certs", e))?;

    // Key
    let key_file = File::open(&config.key_path).map_err(|e| Error::ReadFile("key", e))?;

    let key = private_key(&mut BufReader::new(key_file))
        .map_err(|e| Error::LoadCert("key", e))?
        .unwrap();

    // Root CA
    let root_ca_file = File::open(&config.root_ca_path).map_err(|e| Error::ReadFile("ca", e))?;

    let root_certs = certs(&mut BufReader::new(root_ca_file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(|e| Error::LoadCert("ca certs", e))?;

    let mut root_cert_store = RootCertStore::empty();
    for cert in root_certs {
        root_cert_store
            .add(cert.clone())
            .map_err(Error::RootStore)?;
    }

    Ok(Certificates {
        certificates,
        key,
        root_cert_store,
    })
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("ReadFileError {0}:\n{1}")]
    ReadFile(&'static str, #[source] io::Error),
    #[error("LoadCertError {0}:\n{1}")]
    LoadCert(&'static str, #[source] io::Error),
    #[error("RootStoreError:\n{0}")]
    RootStore(#[source] futures_rustls::rustls::Error),
}
