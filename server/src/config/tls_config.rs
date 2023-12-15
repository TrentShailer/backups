use std::net::SocketAddr;

use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

use super::{
    certificate::{load_certs, load_key, load_roots, CertificateError, KeyError, RootsError},
    RawConfig,
};

pub struct TlsConfig {
    pub socket_address: SocketAddr,
    pub root_ca: RootCertStore,
    pub certificate_chain: Vec<CertificateDer<'static>>,
    pub private_key: PrivateKeyDer<'static>,
}

impl TryFrom<&RawConfig> for TlsConfig {
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        let certificate_chain = load_certs(&value.certificate_path)?;
        let private_key = load_key(&value.private_key_path)?;
        let root_ca = load_roots(&value.root_ca_path)?;

        Ok(Self {
            certificate_chain,
            private_key,
            root_ca,
            socket_address: value.socket_address,
        })
    }

    type Error = ParseTlsConfigError;
}

#[derive(Debug, Error)]
pub enum ParseTlsConfigError {
    #[error("LoadCertsError[br]{0}")]
    LoadCertsError(#[from] CertificateError),
    #[error("LoadPrivateKeyError[br]{0}")]
    LoadPrivateKeyError(#[from] KeyError),
    #[error("LoadRootCAError[br]{0}")]
    LoadRootCAError(#[from] RootsError),
}
