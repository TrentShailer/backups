use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;
use tokio_rustls::rustls::RootCertStore;

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error(transparent)]
    IOError(#[from] io::Error),
}

pub fn load_certs(
    certificate_path: &Path,
) -> Result<Vec<CertificateDer<'static>>, CertificateError> {
    match certs(&mut BufReader::new(File::open(certificate_path)?)).collect() {
        Ok(v) => Ok(v),
        Err(error) => Err(error.into()),
    }
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error("NotFoundError: Key not found")]
    NotFoundError,
}

pub fn load_key(key_path: &Path) -> Result<PrivateKeyDer<'static>, KeyError> {
    let file = File::open(key_path)?;
    let mut reader = BufReader::new(file);

    match private_key(&mut reader)? {
        Some(value) => Ok(value),
        None => Err(KeyError::NotFoundError),
    }
}

#[derive(Debug, Error)]
pub enum RootsError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
}

pub fn load_roots(roots_path: &Path) -> Result<RootCertStore, RootsError> {
    let root_certs: io::Result<Vec<CertificateDer<'static>>> =
        certs(&mut BufReader::new(File::open(roots_path)?)).collect();

    let root_certs = root_certs?;

    let mut root_cert_store = RootCertStore::empty();

    for cert in root_certs {
        root_cert_store.add(cert.clone())?;
    }

    Ok(root_cert_store)
}
