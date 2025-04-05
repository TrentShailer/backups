#![allow(missing_docs)]

use bytemuck::{
    PodCastError, bytes_of,
    checked::{self, CheckedCastError},
};
use shared::{Cadance, Metadata};

pub fn valid_32() -> [u8; 32] {
    Metadata::pad_string(b"u32_byte_string\0")
}

pub fn valid_128() -> [u8; 128] {
    Metadata::pad_string(b"u128_byte_string\0")
}

#[test]
fn valid_string() {
    let _metadata = Metadata::new(0, valid_128(), Cadance::Daily, valid_32());
}

#[test]
fn valid_from_bytes() {
    let metadata = Metadata::new(0, valid_128(), Cadance::Daily, valid_32());
    let bytes = bytes_of(&metadata);
    let _metadata: Metadata = *checked::try_from_bytes(bytes).unwrap();
}

#[test]
#[should_panic]
fn empty_string() {
    let _metadata = Metadata::new(0, Metadata::pad_string(b""), Cadance::Daily, valid_32());
}

#[test]
fn empty_string_from_bytes() {
    let metadata = unsafe {
        Metadata::new_unchecked(0, Metadata::pad_string(b""), Cadance::Daily, valid_32())
    };
    let bytes = bytes_of(&metadata);
    let result = checked::try_from_bytes::<Metadata>(bytes);
    assert!(matches!(result, Err(CheckedCastError::InvalidBitPattern)));
}

#[test]
#[should_panic]
fn nul_string() {
    let _metadata = Metadata::new(0, valid_128(), Cadance::Daily, Metadata::pad_string(b"\0"));
}

#[test]
fn nul_string_from_bytes() {
    let metadata = unsafe {
        Metadata::new_unchecked(0, Metadata::pad_string(b""), Cadance::Daily, valid_32())
    };
    let bytes = bytes_of(&metadata);
    let result = checked::try_from_bytes::<Metadata>(bytes);
    assert!(matches!(result, Err(CheckedCastError::InvalidBitPattern)));
}

#[test]
#[should_panic]
fn invalid_character() {
    let _metadata = Metadata::new(0, Metadata::pad_string(b"#"), Cadance::Daily, valid_32());
}

#[test]
fn invalid_character_from_bytes() {
    let metadata = unsafe {
        Metadata::new_unchecked(0, Metadata::pad_string(b"#"), Cadance::Daily, valid_32())
    };
    let bytes = bytes_of(&metadata);

    let result = checked::try_from_bytes::<Metadata>(bytes);
    assert!(matches!(result, Err(CheckedCastError::InvalidBitPattern)));
}

#[test]
fn too_many_bytes() {
    let bytes = [0u8; size_of::<Metadata>() * 2];
    let result = checked::try_from_bytes::<Metadata>(&bytes);
    assert!(matches!(
        result,
        Err(CheckedCastError::PodCastError(PodCastError::SizeMismatch))
    ),);
}

#[test]
fn too_few_bytes() {
    let bytes = [0u8; 2];
    let result = checked::try_from_bytes::<Metadata>(&bytes);
    assert!(matches!(
        result,
        Err(CheckedCastError::PodCastError(PodCastError::SizeMismatch))
    ),);
}
