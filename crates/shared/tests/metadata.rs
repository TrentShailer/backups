#![allow(missing_docs, non_snake_case)]

use core::{alloc::Layout, mem::offset_of};

use shared::{Cadence, Endian, Metadata, MetadataError, MetadataString, MetadataStringError};

pub fn valid_32() -> MetadataString<32> {
    MetadataString::try_from("32_byte_string").unwrap()
}

pub fn valid_128() -> MetadataString<128> {
    MetadataString::try_from("128_byte_string").unwrap()
}

#[test]
fn Metadata_Layout_HasNoPadding() {
    let fields = [
        Layout::new::<u64>(),
        Layout::new::<MetadataString<128>>(),
        Layout::new::<Cadence>(),
        Layout::new::<MetadataString<32>>(),
        Layout::new::<Endian>(),
        Layout::new::<[u8; 15]>(),
    ];

    let mut layout = unsafe { Layout::from_size_align_unchecked(0, 1) };

    for field in fields {
        let (new_layout, offset) = layout.extend(field).unwrap();

        let padding = offset - layout.size();
        assert_eq!(padding, 0);

        layout = new_layout;
    }

    assert_eq!(
        layout.pad_to_align().size() - layout.size(),
        0,
        "Metadata has padding or test has not been updated"
    );
    assert_eq!(size_of::<Metadata>(), layout.size());
    assert_eq!(align_of::<Metadata>(), layout.align());
}

#[test]
fn TryFromBytes_Valid_IsCorrect() {
    let metadata = Metadata::new(0, valid_128(), Cadence::Daily, valid_32());
    let bytes = metadata.to_bytes();
    let new_metadata = Metadata::try_from(bytes.as_slice()).unwrap();
    assert_eq!(metadata, new_metadata);
}

#[test]
fn TryFromBytes_NullString_IsError() {
    let metadata = unsafe {
        Metadata::new(
            0,
            MetadataString::new_unchecked([0u8; 128]),
            Cadence::Daily,
            valid_32(),
        )
    };
    let bytes = metadata.to_bytes();
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetadataError::InvalidServiceName(MetadataStringError::Invalid(0, b'\0', '\0'))
    );

    // ---

    let metadata = unsafe {
        Metadata::new(
            0,
            valid_128(),
            Cadence::Daily,
            MetadataString::new_unchecked([0u8; 32]),
        )
    };
    let bytes = metadata.to_bytes();
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetadataError::InvalidFileExtension(MetadataStringError::Invalid(0, b'\0', '\0'))
    );
}

#[test]
fn TryFromBytes_InvalidString_IsError() {
    let metadata = unsafe {
        let mut bytes = [0u8; 128];
        bytes[0] = b'#';

        Metadata::new(
            0,
            MetadataString::new_unchecked(bytes),
            Cadence::Daily,
            valid_32(),
        )
    };
    let bytes = metadata.to_bytes();
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetadataError::InvalidServiceName(MetadataStringError::Invalid(0, b'#', '#'))
    );

    // ---

    let metadata = unsafe {
        let mut bytes = [0u8; 32];
        bytes[0] = b'#';

        Metadata::new(
            0,
            valid_128(),
            Cadence::Daily,
            MetadataString::new_unchecked(bytes),
        )
    };
    let bytes = metadata.to_bytes();
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetadataError::InvalidFileExtension(MetadataStringError::Invalid(0, b'#', '#'))
    );
}

#[test]
fn TryFromBytes_InvalidCadance_IsError() {
    let metadata = Metadata::new(0, valid_128(), Cadence::Daily, valid_32());

    let mut bytes = metadata.to_bytes();
    bytes[offset_of!(Metadata, cadence)..offset_of!(Metadata, cadence) + size_of::<u64>()]
        .copy_from_slice(&u64::MAX.to_ne_bytes());

    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(error, MetadataError::InvalidCadance(u64::MAX));
}

#[test]
fn TryFromBytes_TooFewBytes_IsError() {
    let bytes = [0u8; 16];
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(error, MetadataError::WrongSize(16, size_of::<Metadata>()));
}

#[test]
fn TryFromBytes_TooManyBytes_IsError() {
    let bytes = [0u8; size_of::<Metadata>() * 2];
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetadataError::WrongSize(size_of::<Metadata>() * 2, size_of::<Metadata>())
    );
}

#[test]
fn TryFromBytes_InvalidEndian_IsError() {
    let metadata = Metadata::new(0, valid_128(), Cadence::Daily, valid_32());
    let mut bytes = metadata.to_bytes();
    *bytes.get_mut(offset_of!(Metadata, endian)).unwrap() = 3;
    let error = Metadata::try_from(bytes.as_slice()).unwrap_err();
    assert_eq!(error, MetadataError::InvalidEndian(3));
}

#[test]
fn TryFromBytes_OppositeEndian_IsSuccess() {
    let metadata = Metadata::new(10, valid_128(), Cadence::Daily, valid_32());

    let mut bytes = metadata.to_bytes();
    bytes
        [offset_of!(Metadata, backup_bytes)..offset_of!(Metadata, backup_bytes) + size_of::<u64>()]
        .reverse();
    bytes[offset_of!(Metadata, cadence)..offset_of!(Metadata, cadence) + size_of::<u64>()]
        .reverse();
    *bytes.get_mut(offset_of!(Metadata, endian)).unwrap() = if metadata.endian == Endian::Little {
        u8::from(Endian::Big)
    } else {
        u8::from(Endian::Little)
    };

    let new_metadata = Metadata::try_from(bytes.as_slice()).unwrap();
    assert_eq!(new_metadata.cadence, Cadence::Daily);
    assert_eq!(new_metadata.backup_bytes, 10);
    assert_eq!(new_metadata.endian, metadata.endian);
}
