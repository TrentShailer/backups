//! Unit tests for handling client
//!

use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::io::Cursor;

use backup_receiver::Context;
use common::{check_backup_payload, clear_backups, test_receiver};
use shared::{Cadance, Metadata, MetadataString, Response, test::CertificateAuthority};

mod common;

#[test]
fn handle_average_client() {
    let ca = CertificateAuthority::new();
    let mut receiver = test_receiver(&ca);
    let mut context = Context::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let payload = vec![0u8; 512];
    let metadata = Metadata::new(
        512,
        MetadataString::try_from("handle_average_client").unwrap(),
        Cadance::Daily,
        MetadataString::try_from("test").unwrap(),
    );
    clear_backups(&metadata);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(&metadata.as_be_bytes());
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
    let mut context = Context::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let payload = vec![0u8; 256];
    let metadata = Metadata::new(
        512,
        MetadataString::try_from("handle_payload_timeout").unwrap(),
        Cadance::Daily,
        MetadataString::try_from("test").unwrap(),
    );
    clear_backups(&metadata);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(&metadata.as_be_bytes());
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
    let mut context = Context::default();
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
    let mut context = Context::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let data = vec![0u8; size_of::<Metadata>()];
    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);

    assert_eq!(result, Err(Response::BadData), "{:#?}", result);
}
