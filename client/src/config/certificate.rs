use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
    str::FromStr,
};

use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

#[tracing::instrument(skip_all, err)]
pub fn load_cert(
    certificate_path: &PathBuf,
) -> Result<Vec<CertificateDer<'static>>, LoadCertError> {
    let file = File::open(certificate_path).map_err(|e| LoadCertError::OpenFileError(e))?;

    certs(&mut BufReader::new(file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(|e| LoadCertError::ReadCertError(e))
}

#[tracing::instrument(skip_all, err)]
pub fn load_key(key_path: &PathBuf) -> Result<PrivateKeyDer<'static>, LoadKeyError> {
    let file = File::open(key_path.clone()).map_err(|e| LoadKeyError::OpenFileError(e))?;

    let mut reader = BufReader::new(file);

    let maybe_key = private_key(&mut reader).map_err(|e| LoadKeyError::ReadKeyError(e))?;

    match maybe_key {
        Some(value) => Ok(value.into()),
        None => Err(LoadKeyError::KeyNotFoundError),
    }
}

#[tracing::instrument(skip_all, err)]
pub fn load_root_cert(root_path: &PathBuf) -> Result<RootCertStore, LoadRootCertError> {
    let file = File::open(root_path).map_err(|e| LoadRootCertError::OpenFileError(e))?;

    let root_certs = certs(&mut BufReader::new(file))
        .collect::<io::Result<Vec<CertificateDer<'static>>>>()
        .map_err(|e| LoadRootCertError::ReadCertError(e))?;

    let mut root_cert_store = RootCertStore::empty();

    for cert in root_certs {
        root_cert_store
            .add(cert.clone())
            .map_err(|e| LoadRootCertError::AddToStoreError(e))?
    }

    Ok(root_cert_store)
}

#[tracing::instrument(skip_all, err)]
pub fn load_recipiant(
    recipiant_path: &PathBuf,
) -> Result<age::x25519::Recipient, LoadRecipiantError> {
    let contents =
        fs::read_to_string(recipiant_path).map_err(|e| LoadRecipiantError::ReadFileError(e))?;

    match age::x25519::Recipient::from_str(&contents) {
        Ok(v) => Ok(v),
        Err(error) => Err(LoadRecipiantError::ParseError(String::from(error))),
    }
}

#[derive(Debug, Error)]
pub enum LoadCertError {
    #[error("OpenFileError -> {0}")]
    OpenFileError(#[source] io::Error),
    #[error("ReadCertError -> {0}")]
    ReadCertError(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum LoadKeyError {
    #[error("OpenFileError -> {0}")]
    OpenFileError(#[source] io::Error),
    #[error("ParseKeyError -> {0}")]
    ReadKeyError(#[source] io::Error),
    #[error("KeyNotFoundError")]
    KeyNotFoundError,
}

#[derive(Debug, Error)]
pub enum LoadRootCertError {
    #[error("OpenFileError -> {0}")]
    OpenFileError(#[source] io::Error),
    #[error("ReadCertError -> {0}")]
    ReadCertError(#[source] io::Error),
    #[error("AddToStoreError -> {0}")]
    AddToStoreError(#[source] rustls::Error),
}

#[derive(Debug, Error)]
pub enum LoadRecipiantError {
    #[error("ReadFileError -> {0}")]
    ReadFileError(#[source] io::Error),
    #[error("ParseError -> {0}")]
    ParseError(String),
}
