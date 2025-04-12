#![allow(missing_docs)]

use shared::{Cadance, Metadata, MetadataFromBytesError, MetadataString, MetadataStringError};

pub fn valid_32() -> MetadataString<32> {
    MetadataString::try_from("32_byte_string").unwrap()
}

pub fn valid_128() -> MetadataString<128> {
    MetadataString::try_from("128_byte_string").unwrap()
}

#[test]
fn valid_string() {
    let _metadata = Metadata::new(0, valid_128(), Cadance::Daily, valid_32());
}

#[test]
fn valid_from_bytes() {
    let metadata = Metadata::new(0, valid_128(), Cadance::Daily, valid_32());
    let bytes = metadata.as_be_bytes();
    let _metadata = Metadata::try_from_be_bytes(bytes).unwrap();
}

#[test]
fn null_string() {
    let metadata = unsafe {
        Metadata::new(
            0,
            MetadataString::new_unchecked([0u8; 128]),
            Cadance::Daily,
            valid_32(),
        )
    };
    let bytes = metadata.as_be_bytes();
    let result = Metadata::try_from_be_bytes(bytes);
    assert!(matches!(
        result,
        Err(MetadataFromBytesError::InvalidServiceName(
            MetadataStringError::Invalid(0, b'\0', '\0')
        ))
    ));

    // ---

    let metadata = unsafe {
        Metadata::new(
            0,
            valid_128(),
            Cadance::Daily,
            MetadataString::new_unchecked([0u8; 32]),
        )
    };
    let bytes = metadata.as_be_bytes();
    let result = Metadata::try_from_be_bytes(bytes);
    assert!(matches!(
        result,
        Err(MetadataFromBytesError::InvalidFileExtension(
            MetadataStringError::Invalid(0, b'\0', '\0')
        ))
    ));
}

#[test]
fn invalid_string() {
    let metadata = unsafe {
        let mut bytes = [0u8; 128];
        bytes[0] = b'#';

        Metadata::new(
            0,
            MetadataString::new_unchecked(bytes),
            Cadance::Daily,
            valid_32(),
        )
    };
    let bytes = metadata.as_be_bytes();
    let result = Metadata::try_from_be_bytes(bytes);
    assert!(matches!(
        result,
        Err(MetadataFromBytesError::InvalidServiceName(
            MetadataStringError::Invalid(0, b'#', '#')
        ))
    ));

    // ---

    let metadata = unsafe {
        let mut bytes = [0u8; 32];
        bytes[0] = b'#';

        Metadata::new(
            0,
            valid_128(),
            Cadance::Daily,
            MetadataString::new_unchecked(bytes),
        )
    };
    let bytes = metadata.as_be_bytes();
    let result = Metadata::try_from_be_bytes(bytes);
    assert!(matches!(
        result,
        Err(MetadataFromBytesError::InvalidFileExtension(
            MetadataStringError::Invalid(0, b'#', '#')
        ))
    ));
}

#[test]
fn invalid_cadance() {
    let metadata = unsafe {
        Metadata::new(
            0,
            valid_128(),
            core::mem::transmute::<u64, Cadance>(u64::MAX),
            valid_32(),
        )
    };
    let bytes = metadata.as_be_bytes();
    let result = Metadata::try_from_be_bytes(bytes);
    assert!(matches!(
        result,
        Err(MetadataFromBytesError::InvalidCadance(u64::MAX))
    ));
}
