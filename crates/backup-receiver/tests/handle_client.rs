//! Unit tests for handling client
//!

use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::io::Cursor;

use backup_receiver::ContextLogger;
use bytemuck::bytes_of;
use common::{check_backup_payload, clear_backups, test_receiver};
use shared::{Cadance, Metadata, Response, test::CertificateAuthority};

mod common;

#[test]
fn handle_average_client() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let mut context = ContextLogger::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let payload = vec![0u8; 512];
    let metadata = Metadata::new(
        512,
        Metadata::pad_string(b"handle_average_client"),
        Cadance::Daily,
        Metadata::pad_string(b"test"),
    );
    clear_backups(&metadata);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(bytes_of(&metadata));
        data.extend_from_slice(&payload);

        data
    };
    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);

    assert_eq!(result, Ok(metadata), "{:#?}", result);
    check_backup_payload(&metadata, &payload);
    clear_backups(&metadata);
}

#[test]
fn handle_payload_timeout() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    receiver.config.limits.timeout_seconds = 1;
    let mut context = ContextLogger::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let payload = vec![0u8; 256];
    let metadata = Metadata::new(
        512,
        Metadata::pad_string(b"handle_payload_timeout"),
        Cadance::Daily,
        Metadata::pad_string(b"test"),
    );
    clear_backups(&metadata);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(bytes_of(&metadata));
        data.extend_from_slice(&payload);

        data
    };
    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);

    assert_eq!(result, Err(Response::Timeout), "{:#?}", result);
}

#[test]
fn handle_bad_metadata() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let mut context = ContextLogger::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let data = vec![0u8; size_of::<Metadata>() - 8];
    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);

    assert_eq!(result, Err(Response::BadData), "{:#?}", result);
}

#[test]
fn handle_invalid_metadata() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let mut context = ContextLogger::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let data = vec![0u8; size_of::<Metadata>()];
    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);

    assert_eq!(result, Err(Response::BadData), "{:#?}", result);
}
