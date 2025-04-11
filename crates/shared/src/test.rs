//! # test_pki
//! Generate certificates and keys for testing
//!

use core::net::{IpAddr, Ipv4Addr};
use std::io;

use rcgen::{Certificate, KeyPair, SanType};
use rustls::RootCertStore;
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tracing::{
    Level,
    subscriber::{DefaultGuard, set_default},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, registry};

/// A certificate authority for testing.
pub struct CertificateAuthority {
    /// The CA Key
    pub key: KeyPair,
    /// The CA Certificate
    pub certificate: Certificate,
}

impl CertificateAuthority {
    /// Create a new certificate authority.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "CA");
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "CA");
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();
        let certificate = params.self_signed(&key).unwrap();

        Self { key, certificate }
    }

    /// Create a certificate trust store containing the CA.
    pub fn certificate_store(&self) -> RootCertStore {
        let mut roots = RootCertStore::empty();
        roots.add(self.certificate.der().clone()).unwrap();

        roots
    }

    /// Generate a key and sign it's certificate.
    pub fn generate_signed(&self) -> (KeyPair, Certificate) {
        // Create a client end entity cert issued by the CA.
        let mut client_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        client_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Signed");
        client_params.subject_alt_names = vec![SanType::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST))];
        client_params.is_ca = rcgen::IsCa::NoCa;

        client_params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ClientAuth,
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        ];
        client_params.serial_number = Some(rcgen::SerialNumber::from(vec![0xC0, 0xFF, 0xEE]));

        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();
        let certificate = client_params
            .signed_by(&key, &self.certificate, &self.key)
            .unwrap();

        (key, certificate)
    }
}

/// Convert a keypair to a private key der
pub fn private_key_der(key: &KeyPair) -> PrivateKeyDer<'static> {
    PrivateKeyDer::from(PrivatePkcs8KeyDer::from(key.serialize_der()))
}

/// Create and set the global loggers.
pub fn init_test_logger() -> (DefaultGuard, WorkerGuard) {
    let filter = tracing_subscriber::filter::Targets::new().with_default(Level::TRACE);

    // Std layer
    let (std_guard, std_layer) = {
        let (writer, guard) = tracing_appender::non_blocking(io::stdout());

        let layer = tracing_subscriber::fmt::layer()
            .with_writer(writer)
            .with_ansi(true)
            .with_target(false);

        (guard, layer)
    };

    // Create registry
    let registry = registry().with(std_layer).with(filter);

    // Set global subscriber
    let subscriber = set_default(registry);

    (subscriber, std_guard)
}
