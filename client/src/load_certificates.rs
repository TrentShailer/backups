use std::{
    fs::File,
    io::{self, BufReader},
};

use futures_rustls::rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, InvalidDnsNameError, PrivateKeyDer, ServerName};
use thiserror::Error;

use crate::scheduler_config::SchedulerConfig;

pub fn load_certificates(
    config: &SchedulerConfig,
) -> Result<
    (
        Vec<CertificateDer<'static>>,
        PrivateKeyDer<'static>,
        RootCertStore,
        ServerName<'static>,
    ),
    Error,
> {
    let cert_file = File::open(&config.certificate_path).map_err(|e| Error::ReadFile("cert", e))?;
    let certificates: Vec<CertificateDer<'static>> = certs(&mut BufReader::new(cert_file))
        .collect::<io::Result<Vec<_>>>()
        .map_err(|e| Error::LoadCert("certs", e))?;

    let key_file = File::open(&config.key_path).map_err(|e| Error::ReadFile("key", e))?;
    let key = private_key(&mut BufReader::new(key_file))
        .map_err(|e| Error::LoadCert("key", e))?
        .unwrap();

    let root_ca_file = File::open(&config.root_ca_path).map_err(|e| Error::ReadFile("ca", e))?;
    let root_certs: Vec<CertificateDer<'static>> = certs(&mut BufReader::new(root_ca_file))
        .collect::<io::Result<Vec<_>>>()
        .map_err(|e| Error::LoadCert("ca certs", e))?;
    let mut root_cert_store = RootCertStore::empty();
    for cert in root_certs {
        root_cert_store
            .add(cert.clone())
            .map_err(Error::RootStore)?;
    }

    let domain = ServerName::try_from(config.socket_address.clone()).map_err(Error::ServerName)?;

    Ok((certificates, key, root_cert_store, domain))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("ReadFileError {0}:\n{1}")]
    ReadFile(&'static str, #[source] io::Error),
    #[error("LoadCertError {0}:\n{1}")]
    LoadCert(&'static str, #[source] io::Error),
    #[error("RootStoreError:\n{0}")]
    RootStore(#[source] futures_rustls::rustls::Error),
    #[error("ServerNameError:\n{0}")]
    ServerName(#[source] InvalidDnsNameError),
}
