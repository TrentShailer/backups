//! # test_pki
//! Generate certificates and keys for testing
//!

use std::io;

use rcgen::{Certificate, KeyPair};
use rustls::RootCertStore;
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tracing::{Level, subscriber::set_global_default};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, registry};

/// Generate a new certificate authority
pub fn new_certificate_authority() -> ((KeyPair, Certificate), RootCertStore) {
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

    let mut roots = RootCertStore::empty();
    roots.add(certificate.der().clone()).unwrap();

    ((key, certificate), roots)
}

/// Generate a new key and signed certificate
pub fn new_signed(certificate_authority: &(KeyPair, Certificate)) -> (KeyPair, Certificate) {
    // Create a client end entity cert issued by the CA.
    let mut client_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
    client_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Signed");
    client_params.is_ca = rcgen::IsCa::NoCa;
    client_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    client_params.serial_number = Some(rcgen::SerialNumber::from(vec![0xC0, 0xFF, 0xEE]));

    let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).unwrap();
    let certificate = client_params
        .signed_by(&key, &certificate_authority.1, &certificate_authority.0)
        .unwrap();

    (key, certificate)
}

/// Convert a keypair to a private key der
pub fn private_key_der(key: &KeyPair) -> PrivateKeyDer<'static> {
    PrivateKeyDer::from(PrivatePkcs8KeyDer::from(key.serialize_der()))
}

/// Create and set the global loggers.
pub fn init_test_logger() -> WorkerGuard {
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
    set_global_default(registry).unwrap();

    std_guard
}
