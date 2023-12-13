use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;
use tracing::debug;

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
    #[tracing::instrument(level = "trace", skip_all, err)]
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        debug!("Started parsing tls config");

        let certificate = load_cert(&value.certificate_path)?;
        debug!("Loaded certificate");

        let key = load_key(&value.key_path)?;
        debug!("Loaded key");

        let root_ca = load_root_cert(&value.root_ca_path)?;
        debug!("Loaded root cert");

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
    #[error("LoadCertError -> {0}")]
    LoadCertError(#[from] LoadCertError),
    #[error("LoadKeyError -> {0}")]
    LoadKeyError(#[from] LoadKeyError),
    #[error("LoadRootCertError -> {0}")]
    LoadRootCertError(#[from] LoadRootCertError),
}
