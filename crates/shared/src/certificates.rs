use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use rustls::RootCertStore;
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use thiserror::Error;

/// Structure containing certificates for mTLS.
pub struct Certificates {
    /// The certificate chain.
    pub certificate_chain: Vec<CertificateDer<'static>>,

    /// The private key for the certificate.
    pub private_key: PrivateKeyDer<'static>,

    /// The trust store.
    pub trust_store: RootCertStore,
}

impl Certificates {
    /// Tries to load the certificates from given paths.
    pub fn load(
        root_certificate_file: &Path,
        certificate_file: &Path,
        private_key_file: &Path,
    ) -> Result<Self, CertificateError> {
        // Load the certificate chain.
        let certificate_chain = {
            let cert_file =
                File::open(certificate_file).map_err(CertificateError::LoadCertificate)?;

            let certificate_chain: Vec<_> = certs(&mut BufReader::new(cert_file))
                .collect::<io::Result<_>>()
                .map_err(CertificateError::LoadCertificate)?;

            if certificate_chain.is_empty() {
                return Err(CertificateError::NoCertificate);
            }

            certificate_chain
        };

        // Load the private key.
        let private_key = {
            let private_key_file =
                File::open(private_key_file).map_err(CertificateError::LoadPrivateKey)?;

            private_key(&mut BufReader::new(private_key_file))
                .map_err(CertificateError::LoadPrivateKey)?
                .ok_or(CertificateError::NoPrivateKey)?
        };

        // Create the trust store.
        let trust_store = {
            // Load the root certificates.
            let root_certificate_file =
                File::open(root_certificate_file).map_err(CertificateError::LoadRootCertificate)?;
            let root_certificates: Vec<_> = certs(&mut BufReader::new(root_certificate_file))
                .collect::<io::Result<_>>()
                .map_err(CertificateError::LoadRootCertificate)?;

            if root_certificates.is_empty() {
                return Err(CertificateError::NoRootCertificate);
            }

            // Create the trust store
            let mut trust_store = RootCertStore::empty();
            for cert in root_certificates {
                trust_store.add(cert.clone())?;
            }

            trust_store
        };

        Ok(Self {
            certificate_chain,
            private_key,
            trust_store,
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CertificateError {
    #[error("Failed to load the certificate file\n{0}")]
    LoadCertificate(#[source] io::Error),

    #[error("The certificate file contained no certificates.")]
    NoCertificate,

    #[error("Failed to load the private key file\n{0}")]
    LoadPrivateKey(#[source] io::Error),

    #[error("The private key file contained no private keys.")]
    NoPrivateKey,

    #[error("Failed to load the root certificate file\n{0}")]
    LoadRootCertificate(#[source] io::Error),

    #[error("The root certificate file contained no certificates.")]
    NoRootCertificate,

    #[error("Failed to create the trust store\n{0}")]
    CreateTrustStore(#[from] rustls::Error),
}
