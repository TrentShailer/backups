use std::{fs::File, io::BufReader, path::Path};

use blake3::Hash;
use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct TlsPayload {
    pub file_size: usize,
    pub file_hash: Hash,
    pub file_name: String,
    pub service_name: String,
    pub backup_name: String,
    pub max_files: usize,
}

pub struct Certificates {
    pub certificates: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub root_cert_store: RootCertStore,
}

pub fn load_certificates(
    root_ca_path: &Path,
    certificate_path: &Path,
    key_path: &Path,
) -> Result<Certificates, LoadCertsError> {
    // Cert
    let cert_file = File::open(certificate_path)?;
    let certificates: Vec<CertificateDer<'_>> =
        certs(&mut BufReader::new(cert_file)).collect::<std::io::Result<_>>()?;

    // Key
    let key_file = File::open(key_path)?;
    let key = private_key(&mut BufReader::new(key_file))?.unwrap();

    // Root CA
    let root_ca_file = File::open(root_ca_path)?;
    let root_certs: Vec<CertificateDer<'_>> =
        certs(&mut BufReader::new(root_ca_file)).collect::<std::io::Result<_>>()?;

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
pub enum LoadCertsError {
    #[error("IoError:\n{0}")]
    Io(#[from] std::io::Error),
    #[error("RootStoreError:\n{0}")]
    RootStore(#[from] rustls::Error),
}
