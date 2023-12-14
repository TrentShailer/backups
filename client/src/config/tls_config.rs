use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

use crate::config::certificate::{load_cert, load_key, load_root_cert};

use super::{
    certificate::{LoadCertError, LoadKeyError, LoadRootCertError},
    raw_config::RawConfig,
};

pub struct TlsConfig {
    pub address: String,
    pub port: u16,
    pub root_ca: RootCertStore,
    pub certificate: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
}

impl TryFrom<&RawConfig> for TlsConfig {
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        let certificate = load_cert(&value.certificate_path)?;

        let key = load_key(&value.key_path)?;

        let root_ca = load_root_cert(&value.root_ca_path)?;

        Ok(Self {
            address: value.socket_address.clone(),
            port: value.socket_port,
            root_ca,
            certificate,
            key,
        })
    }
    type Error = ParseTlsConfigError;
}

#[derive(Debug, Error)]
pub enum ParseTlsConfigError {
    #[error("LoadCertError\n{0}")]
    LoadCertError(#[from] LoadCertError),
    #[error("LoadKeyError\n{0}")]
    LoadKeyError(#[from] LoadKeyError),
    #[error("LoadRootCertError\n{0}")]
    LoadRootCertError(#[from] LoadRootCertError),
}
