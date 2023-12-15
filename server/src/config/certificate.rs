use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
    str::FromStr,
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
    certificate_path: &PathBuf,
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

pub fn load_key(key_path: &PathBuf) -> Result<PrivateKeyDer<'static>, KeyError> {
    let file = File::open(key_path.clone())?;
    let mut reader = BufReader::new(file);

    match private_key(&mut reader)? {
        Some(value) => Ok(value.into()),
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

pub fn load_roots(roots_path: &PathBuf) -> Result<RootCertStore, RootsError> {
    let root_certs: io::Result<Vec<CertificateDer<'static>>> =
        certs(&mut BufReader::new(File::open(roots_path)?)).collect();

    let root_certs = root_certs?;

    let mut root_cert_store = RootCertStore::empty();

    for cert in root_certs {
        root_cert_store.add(cert.clone())?;
    }

    Ok(root_cert_store)
}

#[derive(Debug, Error)]
pub enum AgeKeyError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error("{0}")]
    ParseError(String),
}

pub fn load_age_key(age_key_path: &PathBuf) -> Result<age::x25519::Identity, AgeKeyError> {
    let contents = fs::read_to_string(age_key_path)?;
    match age::x25519::Identity::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(error) => Err(AgeKeyError::ParseError(String::from(error))),
    }
}
