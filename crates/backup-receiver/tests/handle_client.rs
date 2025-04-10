//! Unit tests for handling client
//!

use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::io::Cursor;

use backup_receiver::ContextLogger;
use bytemuck::bytes_of;
use common::test_receiver;
use shared::{Cadance, Metadata, test::init_test_logger};

mod common;

#[test]
fn average_client() {
    let _logger = init_test_logger();
    let mut receiver = test_receiver();

    let mut context = ContextLogger::default();
    let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));

    let metadata = Metadata::new(
        512,
        Metadata::pad_string(b"Test"),
        Cadance::Daily,
        Metadata::pad_string(b"test"),
    );

    let payload = vec![0u8; 512];

    let data = {
        let mut data: Vec<u8> = Vec::new();

        data.extend_from_slice(bytes_of(&metadata));
        data.extend_from_slice(&payload);

        data
    };

    let mut reader = Cursor::new(data);

    let result = receiver.handle_client(&mut context, &mut reader, peer);
    assert!(result.is_ok(), "{:#?}", result);

    let returned_metadata = result.unwrap();

    assert_eq!(returned_metadata, metadata, "{:#?}", returned_metadata);
}
