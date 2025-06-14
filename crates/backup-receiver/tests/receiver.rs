//! Receiver tests
//!

use std::{
    io::{self, Read, Write},
    thread,
};

use common::{check_backup_payload, clear_backups, test_client, test_receiver};
use rustls::{AlertDescription, Stream};
use shared::{Cadence, Metadata, MetadataString, Response, test::CertificateAuthority};

mod common;

#[test]
fn average_client() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let receiver_address = receiver.listener.local_addr().unwrap();
    let thread = thread::spawn(move || {
        receiver.accept_and_handle_client();
    });

    let (client_key, client_cert) = ca.generate_signed();
    let (mut socket, mut client) = test_client(
        client_key,
        client_cert,
        ca.certificate_store(),
        receiver_address,
    );
    let mut stream = Stream::new(&mut client, &mut socket);

    let payload = vec![0u8; 512];
    let metadata = Metadata::new(
        512,
        MetadataString::try_from("average_client").unwrap(),
        Cadence::Daily,
        MetadataString::try_from("test").unwrap(),
    );
    clear_backups(&metadata);

    stream.write_all(&metadata.to_bytes()).unwrap();
    stream.write_all(&payload).unwrap();
    stream.flush().unwrap();
    let mut response_buffer = [0u8; size_of::<Response>()];
    stream.read_exact(&mut response_buffer).unwrap();
    stream.conn.send_close_notify();
    stream.conn.complete_io(stream.sock).unwrap();

    thread.join().unwrap();

    let response = Response::try_from_u64(u64::from_be_bytes(response_buffer)).unwrap();

    assert_eq!(response, Response::Success);
    check_backup_payload(&metadata, &payload);
    clear_backups(&metadata);
}

#[test]
fn untrusted_client() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let receiver_address = receiver.listener.local_addr().unwrap();

    thread::spawn(move || {
        receiver.accept_and_handle_client();
    });

    let client_ca = CertificateAuthority::new();
    let (client_key, client_cert) = client_ca.generate_signed();
    let (mut socket, mut client) = test_client(
        client_key,
        client_cert,
        ca.certificate_store(),
        receiver_address,
    );
    let stream = Stream::new(&mut client, &mut socket);

    let error = stream.conn.complete_io(stream.sock).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    let inner_error: Box<rustls::Error> = error.into_inner().unwrap().downcast().unwrap();
    assert_eq!(
        inner_error,
        Box::new(rustls::Error::AlertReceived(AlertDescription::DecryptError))
    );
}

#[test]
fn short_payload() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    receiver.config.limits.timeout_seconds = 1;
    let receiver_address = receiver.listener.local_addr().unwrap();
    let thread = thread::spawn(move || {
        receiver.accept_and_handle_client();
    });

    let (client_key, client_cert) = ca.generate_signed();
    let (mut socket, mut client) = test_client(
        client_key,
        client_cert,
        ca.certificate_store(),
        receiver_address,
    );
    let mut stream = Stream::new(&mut client, &mut socket);

    let payload = vec![0u8; 256];
    let metadata = Metadata::new(
        512,
        MetadataString::try_from("short_payload").unwrap(),
        Cadence::Daily,
        MetadataString::try_from("test").unwrap(),
    );
    clear_backups(&metadata);

    stream.write_all(&metadata.to_bytes()).unwrap();
    stream.write_all(&payload).unwrap();
    stream.flush().unwrap();
    let mut response_buffer = [0u8; size_of::<Response>()];
    stream.read_exact(&mut response_buffer).unwrap();
    stream.conn.send_close_notify();
    stream.conn.complete_io(stream.sock).unwrap();

    thread.join().unwrap();

    let response = Response::try_from_u64(u64::from_be_bytes(response_buffer)).unwrap();

    assert_eq!(response, Response::Timeout);
    clear_backups(&metadata);
}

#[test]
fn short_metadata() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    receiver.config.limits.timeout_seconds = 1;
    let receiver_address = receiver.listener.local_addr().unwrap();
    let thread = thread::spawn(move || {
        receiver.accept_and_handle_client();
    });

    let (client_key, client_cert) = ca.generate_signed();
    let (mut socket, mut client) = test_client(
        client_key,
        client_cert,
        ca.certificate_store(),
        receiver_address,
    );
    let mut stream = Stream::new(&mut client, &mut socket);

    let metadata = vec![0u8; size_of::<Metadata>() - 8];

    stream.write_all(&metadata).unwrap();
    stream.flush().unwrap();
    let mut response_buffer = [0u8; size_of::<Response>()];
    stream.read_exact(&mut response_buffer).unwrap();
    stream.conn.send_close_notify();
    stream.conn.complete_io(stream.sock).unwrap();

    thread.join().unwrap();

    let response = Response::try_from_u64(u64::from_be_bytes(response_buffer)).unwrap();

    assert_eq!(response, Response::Timeout);
}

#[test]
fn bad_metadata() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    receiver.config.limits.timeout_seconds = 1;
    let receiver_address = receiver.listener.local_addr().unwrap();

    let thread = thread::spawn(move || {
        receiver.accept_and_handle_client();
    });

    let (client_key, client_cert) = ca.generate_signed();
    let (mut socket, mut client) = test_client(
        client_key,
        client_cert,
        ca.certificate_store(),
        receiver_address,
    );
    let mut stream = Stream::new(&mut client, &mut socket);

    let metadata = vec![0u8; size_of::<Metadata>()];
    stream.write_all(&metadata).unwrap();
    stream.flush().unwrap();
    let mut response_buffer = [0u8; size_of::<Response>()];
    stream.read_exact(&mut response_buffer).unwrap();
    stream.conn.send_close_notify();
    stream.conn.complete_io(stream.sock).unwrap();

    thread.join().unwrap();

    let response = Response::try_from_u64(u64::from_be_bytes(response_buffer)).unwrap();

    assert_eq!(response, Response::BadData);
}
