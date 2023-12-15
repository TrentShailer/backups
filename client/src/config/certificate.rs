use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

pub fn load_cert(
    certificate_path: &PathBuf,
) -> Result<Vec<CertificateDer<'static>>, LoadCertError> {
    let file = File::open(certificate_path).map_err(LoadCertError::OpenFileError)?;

    certs(&mut BufReader::new(file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(LoadCertError::ReadCertError)
}

pub fn load_key(key_path: &Path) -> Result<PrivateKeyDer<'static>, LoadKeyError> {
    let file = File::open(key_path).map_err(LoadKeyError::OpenFileError)?;

    let mut reader = BufReader::new(file);

    let maybe_key = private_key(&mut reader).map_err(LoadKeyError::ReadKeyError)?;

    match maybe_key {
        Some(value) => Ok(value),
        None => Err(LoadKeyError::KeyNotFoundError),
    }
}

pub fn load_root_cert(root_path: &PathBuf) -> Result<RootCertStore, LoadRootCertError> {
    let file = File::open(root_path).map_err(LoadRootCertError::OpenFileError)?;

    let root_certs = certs(&mut BufReader::new(file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(LoadRootCertError::ReadCertError)?;

    let mut root_cert_store = RootCertStore::empty();

    for cert in root_certs {
        root_cert_store
            .add(cert.clone())
            .map_err(LoadRootCertError::AddToStoreError)?
    }

    Ok(root_cert_store)
}

pub fn load_recipiant(
    recipiant_path: &PathBuf,
) -> Result<age::x25519::Recipient, LoadRecipiantError> {
    let contents = fs::read_to_string(recipiant_path).map_err(LoadRecipiantError::ReadFileError)?;

    match age::x25519::Recipient::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(error) => Err(LoadRecipiantError::ParseError(String::from(error))),
    }
}

#[derive(Debug, Error)]
pub enum LoadCertError {
    #[error("OpenFileError[br]{0}")]
    OpenFileError(#[source] io::Error),
    #[error("ReadCertError[br]{0}")]
    ReadCertError(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum LoadKeyError {
    #[error("OpenFileError[br]{0}")]
    OpenFileError(#[source] io::Error),
    #[error("ParseKeyError[br]{0}")]
    ReadKeyError(#[source] io::Error),
    #[error("KeyNotFoundError")]
    KeyNotFoundError,
}

#[derive(Debug, Error)]
pub enum LoadRootCertError {
    #[error("OpenFileError[br]{0}")]
    OpenFileError(#[source] io::Error),
    #[error("ReadCertError[br]{0}")]
    ReadCertError(#[source] io::Error),
    #[error("AddToStoreError[br]{0}")]
    AddToStoreError(#[source] rustls::Error),
}

#[derive(Debug, Error)]
pub enum LoadRecipiantError {
    #[error("ReadFileError[br]{0}")]
    ReadFileError(#[source] io::Error),
    #[error("ParseError[br]{0}")]
    ParseError(String),
}
