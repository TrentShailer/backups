use std::sync::Arc;

use rcgen::{Certificate, KeyPair, SignatureAlgorithm};
use rustls::RootCertStore;

/// Test PKI with a ca, client, and server certificate.
pub struct TestPki {
    pub roots: Arc<RootCertStore>,
    #[allow(unused)]
    pub ca_cert: rcgen::CertifiedKey,
    pub client_cert: rcgen::CertifiedKey,
    pub server_cert: rcgen::CertifiedKey,
}

impl TestPki {
    pub fn new_trusted_client() -> Self {
        let algorithm = &rcgen::PKCS_ECDSA_P256_SHA256;

        let ca = Self::gen_ca(algorithm);
        let (ee_key, server_cert) = Self::gen_server_cert(algorithm, &ca);
        let (client_key, client_cert) = Self::gen_trusted_client(algorithm, &ca);

        // Create a root cert store that includes the CA certificate.
        let mut roots = RootCertStore::empty();
        roots.add(ca.1.der().clone()).unwrap();
        Self {
            roots: roots.into(),
            ca_cert: rcgen::CertifiedKey {
                cert: ca.1,
                key_pair: ca.0,
            },
            client_cert: rcgen::CertifiedKey {
                cert: client_cert,
                key_pair: client_key,
            },
            server_cert: rcgen::CertifiedKey {
                cert: server_cert,
                key_pair: ee_key,
            },
        }
    }

    pub fn new_untrusted_client() -> Self {
        let algorithm = &rcgen::PKCS_ECDSA_P256_SHA256;

        let ca = Self::gen_ca(algorithm);
        let (ee_key, server_cert) = Self::gen_server_cert(algorithm, &ca);
        let (client_key, client_cert) = Self::gen_untrusted_client(algorithm);

        // Create a root cert store that includes the CA certificate.
        let mut roots = RootCertStore::empty();
        roots.add(ca.1.der().clone()).unwrap();
        Self {
            roots: roots.into(),
            ca_cert: rcgen::CertifiedKey {
                cert: ca.1,
                key_pair: ca.0,
            },
            client_cert: rcgen::CertifiedKey {
                cert: client_cert,
                key_pair: client_key,
            },
            server_cert: rcgen::CertifiedKey {
                cert: server_cert,
                key_pair: ee_key,
            },
        }
    }

    fn gen_ca(algorithm: &'static SignatureAlgorithm) -> (KeyPair, Certificate) {
        let mut ca_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        ca_params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "Rustls Server Acceptor");

        ca_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Example CA");
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        let ca_key = KeyPair::generate_for(algorithm).unwrap();
        let ca_cert = ca_params.self_signed(&ca_key).unwrap();

        (ca_key, ca_cert)
    }

    fn gen_server_cert(
        algorithm: &'static SignatureAlgorithm,
        ca: &(KeyPair, Certificate),
    ) -> (KeyPair, Certificate) {
        // Create a server end entity cert issued by the CA.
        let mut server_ee_params =
            rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).unwrap();
        server_ee_params.is_ca = rcgen::IsCa::NoCa;
        server_ee_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        let ee_key = KeyPair::generate_for(algorithm).unwrap();
        let server_cert = server_ee_params.signed_by(&ee_key, &ca.1, &ca.0).unwrap();

        (ee_key, server_cert)
    }

    fn gen_trusted_client(
        algorithm: &'static SignatureAlgorithm,
        ca: &(KeyPair, Certificate),
    ) -> (KeyPair, Certificate) {
        // Create a client end entity cert issued by the CA.
        let mut client_ee_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        client_ee_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Example Client");
        client_ee_params.is_ca = rcgen::IsCa::NoCa;
        client_ee_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        client_ee_params.serial_number = Some(rcgen::SerialNumber::from(vec![0xC0, 0xFF, 0xEE]));
        let client_key = KeyPair::generate_for(algorithm).unwrap();
        let client_cert = client_ee_params
            .signed_by(&client_key, &ca.1, &ca.0)
            .unwrap();

        (client_key, client_cert)
    }

    fn gen_untrusted_client(algorithm: &'static SignatureAlgorithm) -> (KeyPair, Certificate) {
        // Create a selfsigned client end entity cert
        let mut client_ee_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
        client_ee_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Example Client");
        client_ee_params.is_ca = rcgen::IsCa::NoCa;
        client_ee_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        client_ee_params.serial_number = Some(rcgen::SerialNumber::from(vec![0xC0, 0xFF, 0xEE]));
        let client_key = KeyPair::generate_for(algorithm).unwrap();
        let client_cert = client_ee_params.self_signed(&client_key).unwrap();

        (client_key, client_cert)
    }
}
