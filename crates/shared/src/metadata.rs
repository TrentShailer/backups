use std::path::PathBuf;

use thiserror::Error;

use crate::{Cadence, Endian, MetadataString, MetadataStringError};

/// Metadata containing information about the backup payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Backup size in bytes.
    pub backup_bytes: u64,

    /// The name of the service this backup is for.
    pub service_name: MetadataString<128>,

    /// The cadence of this backup
    pub cadence: Cadence,

    /// The file extension for the backup.
    pub file_extension: MetadataString<32>,

    /// The endian of the numbers in the struct.
    pub endian: Endian,

    /// Padding to ensure remaining memory is not uninitialised for Metadata.
    padding: [u8; 15],
}

impl Metadata {
    /// Creates a new metadata instance.
    pub fn new(
        backup_bytes: u64,
        service_name: MetadataString<128>,
        cadence: Cadence,
        file_extension: MetadataString<32>,
    ) -> Self {
        Self {
            backup_bytes,
            service_name,
            cadence,
            file_extension,
            endian: Endian::current(),
            padding: [0u8; 15],
        }
    }

    /// Returns the path this backup's output directory.
    pub fn backup_directory(&self) -> PathBuf {
        PathBuf::from("backups")
            .join(self.service_name.as_string())
            .join(self.cadence.as_path())
    }

    /// Converts self to underlying bytes.
    pub fn to_bytes(self) -> [u8; size_of::<Self>()] {
        unsafe { core::mem::transmute::<Self, [u8; size_of::<Self>()]>(self) }
    }
}

impl TryFrom<&[u8]> for Metadata {
    type Error = MetadataError;

    /// Requires that `value` is exactly `size_of::<Self>()`.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // Metadata but using the underlying integer representations of the enums to validate them
        // without potentially triggering undefined behaviour.
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct SafeMetadata {
            pub backup_bytes: u64,
            pub service_name: MetadataString<128>,
            pub cadence: u64,
            pub file_extension: MetadataString<32>,
            pub endian: u8,
            pub padding: [u8; 15],
        }

        let exact_bytes: [u8; size_of::<SafeMetadata>()] = value
            .try_into()
            .map_err(|_| MetadataError::WrongSize(size_of_val(value), size_of::<Self>()))?;

        // Interpret the bytes as an instance of `SafeMetadata`
        let mut unverified_value: SafeMetadata = unsafe { core::mem::transmute(exact_bytes) };

        // Verify the `endian` field.
        let value_endian = Endian::try_from_u8(unverified_value.endian)
            .ok_or(MetadataError::InvalidEndian(unverified_value.endian))?;

        // Convert the value to native endian if required.
        if !value_endian.is_current() {
            unverified_value.backup_bytes = unverified_value.backup_bytes.swap_bytes();
            unverified_value.cadence = unverified_value.cadence.swap_bytes();
            unverified_value.endian = u8::from(Endian::current());
        }

        // Validate remaining fields
        MetadataString::<128>::validate_bytes(unverified_value.service_name.as_bytes())
            .map_err(MetadataError::InvalidServiceName)?;

        MetadataString::<32>::validate_bytes(unverified_value.file_extension.as_bytes())
            .map_err(MetadataError::InvalidFileExtension)?;

        if !Cadence::is_valid(unverified_value.cadence) {
            return Err(MetadataError::InvalidCadance(unverified_value.cadence));
        }

        // All fields uphold the invariants of Metadata, conversion is safe.
        Ok(unsafe { core::mem::transmute::<SafeMetadata, Self>(unverified_value) })
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MetadataError {
    #[error("Invalid service name: {0}")]
    InvalidServiceName(#[source] MetadataStringError),

    #[error("Invalid file extension: {0}")]
    InvalidFileExtension(#[source] MetadataStringError),

    #[error("Invalid cadance: {0}")]
    InvalidCadance(u64),

    #[error("Invalid endian (should be 0 or 1): {0}")]
    InvalidEndian(u8),

    #[error("Source is the wrong size: {0}/{1}")]
    WrongSize(usize, usize),
}
