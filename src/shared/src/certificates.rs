use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

/// Structure containing relevant certificates for mTLS.
pub struct Certificates {
    pub certificates: Vec<CertificateDer<'static>>,
    pub key: PrivateKeyDer<'static>,
    pub root_cert_store: RootCertStore,
}

impl Certificates {
    /// Tries to load the certificates from given paths.
    pub fn load(
        root_ca_path: &Path,
        certificate_path: &Path,
        key_path: &Path,
    ) -> Result<Self, Error> {
        // Cert
        let cert_file = File::open(certificate_path).map_err(Error::Certificate)?;
        let certificates: Vec<CertificateDer<'_>> = certs(&mut BufReader::new(cert_file))
            .collect::<std::io::Result<_>>()
            .map_err(Error::Certificate)?;

        // Key
        let key_file = File::open(key_path).map_err(Error::Key)?;
        let key = private_key(&mut BufReader::new(key_file))
            .map_err(Error::Key)?
            .expect("private_key was none");

        // Root CA
        let root_ca_file = File::open(root_ca_path).map_err(Error::Roots)?;
        let root_certs: Vec<CertificateDer<'_>> = certs(&mut BufReader::new(root_ca_file))
            .collect::<std::io::Result<_>>()
            .map_err(Error::Roots)?;

        let mut root_cert_store = RootCertStore::empty();
        for cert in root_certs {
            root_cert_store.add(cert.clone())?;
        }

        Ok(Self {
            certificates,
            key,
            root_cert_store,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to load vertificate\n{0}")]
    Certificate(#[source] io::Error),

    #[error("Failed to load key\n{0}")]
    Key(#[source] io::Error),

    #[error("Failed to load root certs\n{0}")]
    Roots(#[source] io::Error),

    #[error("Failed to create root cert store\n{0}")]
    RootStore(#[from] rustls::Error),
}
