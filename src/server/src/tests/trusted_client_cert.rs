use std::{fs, net::SocketAddr, path::Path, thread};

use crate::server::Server;

use super::{client::TestClient, TestPki};

struct CleanupBackup;

impl Drop for CleanupBackup {
    fn drop(&mut self) {
        if Path::new("backups/backups_server/tests/trusted_client_cert").exists() {
            fs::remove_file("backups/backups_server/tests/trusted_client_cert")
                .expect("Failed to remove file");
        }
    }
}

#[test]
fn trusted_client_cert() {
    let _cleanup = CleanupBackup;

    let pki = TestPki::new_trusted_client();
    let address: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let mut server = Server::new_test(&pki, &address).expect("Failed to create server");

    let handle = thread::spawn(move || server.accept_blocking());

    let client = TestClient::new(&pki, address);

    let client_result = client.try_make_backup("trusted_client_cert");
    let server_result = handle.join().unwrap();

    server_result.expect("Server returned an error");
    client_result.expect("Client returned an error");
}
