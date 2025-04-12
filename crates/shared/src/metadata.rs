use core::mem::offset_of;
use std::path::PathBuf;

use thiserror::Error;

use crate::{Cadance, MetadataString, MetadataStringError, cadance};

/// Metadata containing information about the backup payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Backup size in bytes.
    pub backup_bytes: u64,

    /// The name of the service this backup is for.
    pub service_name: MetadataString<128>,

    /// The cadance of this backup
    pub cadance: Cadance,

    /// The file extension for the backup.
    pub file_extension: MetadataString<32>,
}

impl Metadata {
    /// Creates a new metadata instance.
    pub fn new(
        backup_bytes: u64,
        service_name: MetadataString<128>,
        cadance: Cadance,
        file_extension: MetadataString<32>,
    ) -> Self {
        Self {
            backup_bytes,
            service_name,
            cadance,
            file_extension,
        }
    }

    /// Returns the path this backup's output directory.
    pub fn backup_directory(&self) -> PathBuf {
        PathBuf::from("backups")
            .join(self.service_name.as_string())
            .join(self.cadance.as_path())
    }

    pub fn as_be_bytes(&self) -> [u8; size_of::<Self>()] {
        let mut bytes = [0u8; size_of::<Self>()];

        {
            const OFFSET: usize = offset_of!(Metadata, backup_bytes);
            const END: usize = OFFSET + size_of::<u64>();
            bytes[OFFSET..END].copy_from_slice(&self.backup_bytes.to_be_bytes());
        }

        {
            const OFFSET: usize = offset_of!(Metadata, service_name);
            const END: usize = OFFSET + size_of::<MetadataString<128>>();
            bytes[OFFSET..END].copy_from_slice(self.service_name.as_bytes());
        }

        {
            const OFFSET: usize = offset_of!(Metadata, cadance);
            const END: usize = OFFSET + size_of::<Cadance>();
            bytes[OFFSET..END].copy_from_slice(&self.cadance.to_be_bytes());
        }

        {
            const OFFSET: usize = offset_of!(Metadata, file_extension);
            const END: usize = OFFSET + size_of::<MetadataString<32>>();
            bytes[OFFSET..END].copy_from_slice(self.file_extension.as_bytes());
        }

        bytes
    }

    pub fn try_from_be_bytes(
        bytes: [u8; size_of::<Self>()],
    ) -> Result<Metadata, MetadataFromBytesError> {
        let backup_bytes = {
            const OFFSET: usize = offset_of!(Metadata, backup_bytes);
            const SIZE: usize = size_of::<u64>();
            const END: usize = OFFSET + SIZE;
            let bytes: [u8; SIZE] = bytes[OFFSET..END].try_into().unwrap();

            u64::from_be_bytes(bytes)
        };

        let service_name = {
            const OFFSET: usize = offset_of!(Metadata, service_name);
            const SIZE: usize = size_of::<MetadataString<128>>();
            const END: usize = OFFSET + SIZE;
            let bytes: [u8; SIZE] = bytes[OFFSET..END].try_into().unwrap();

            MetadataString::<128>::try_from(bytes.as_slice())
                .map_err(MetadataFromBytesError::InvalidServiceName)?
        };

        let cadance = {
            const OFFSET: usize = offset_of!(Metadata, cadance);
            const SIZE: usize = size_of::<Cadance>();
            const END: usize = OFFSET + SIZE;
            let bytes: [u8; SIZE] = bytes[OFFSET..END].try_into().unwrap();

            let value = u64::from_be_bytes(bytes);

            match Cadance::try_from_u64(value) {
                Some(cadance) => cadance,
                None => return Err(MetadataFromBytesError::InvalidCadance(value)),
            }
        };

        let file_extension = {
            const OFFSET: usize = offset_of!(Metadata, file_extension);
            const SIZE: usize = size_of::<MetadataString<32>>();
            const END: usize = OFFSET + SIZE;
            let bytes: [u8; SIZE] = bytes[OFFSET..END].try_into().unwrap();

            MetadataString::<32>::try_from(bytes.as_slice())
                .map_err(MetadataFromBytesError::InvalidFileExtension)?
        };

        Ok(Self {
            backup_bytes,
            service_name,
            cadance,
            file_extension,
        })
    }
}

#[derive(Debug, Error)]
pub enum MetadataFromBytesError {
    #[error("Invalid service name: {0}")]
    InvalidServiceName(#[source] MetadataStringError),

    #[error("Invalid file extension: {0}")]
    InvalidFileExtension(#[source] MetadataStringError),

    #[error("Invalid cadance: {0}")]
    InvalidCadance(u64),
}
