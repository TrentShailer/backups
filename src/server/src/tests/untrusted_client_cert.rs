use std::{io::ErrorKind, net::SocketAddr, thread};

use rustls::CertificateError;

use crate::server::{accept, Server};

use super::{client::TestClient, TestPki};

#[test]
fn untrusted_client_cert() {
    let pki = TestPki::new_untrusted_client();
    let address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut server = Server::new_test(&pki, &address).expect("Failed to create server");

    let handle = thread::spawn(move || server.accept_blocking());

    let client = TestClient::new(&pki, address);

    let _client_result = client.try_make_backup("untrusted_client_cert");
    let server_result = handle.join().unwrap();

    match server_result {
        Ok(_) => panic!("Server did not reject client"),
        Err(e) => {
            let io_err = match &e {
                accept::Error::CompleteHandshake(io_err) => io_err,
                _ => panic!("Unexpected server error:\n{e}"),
            };

            let dyn_io_inner_err = match io_err.kind() {
                ErrorKind::InvalidData => io_err.get_ref().unwrap(),
                _ => panic!("Unexpected server error:\n{e}"),
            };

            let rustls_err: &rustls::Error = dyn_io_inner_err.downcast_ref().unwrap();
            let cert_err = match rustls_err {
                rustls::Error::InvalidCertificate(cert_err) => cert_err,
                _ => panic!("Unexpected server error:\n{e}"),
            };

            if *cert_err != CertificateError::UnknownIssuer {
                panic!("Unexpected server error:\n{e}")
            }
        }
    }
}
