use std::{borrow::Cow, path::PathBuf};

use bytemuck::{CheckedBitPattern, NoUninit};

use crate::Cadance;

/// Metadata containing information about the backup payload.
#[repr(C)]
#[derive(Clone, Copy, Debug, NoUninit, PartialEq, Eq)]
pub struct Metadata {
    /// Backup size in bytes.
    pub backup_bytes: u64,

    /// The name of the service this backup is for. Only `[a-zA-Z0-9_\-\0]` is valid.
    service_name: [u8; 128],

    /// The cadance of this backup
    pub cadance: Cadance,

    /// The file extension for the backup. Only `[a-zA-Z0-9_\-\0]` is valid.
    file_extension: [u8; 32],
}

impl Metadata {
    /// Creates a new metadata instance.
    ///
    /// # Panics
    /// * If `service_name` or `file_extension` are invalid.
    pub fn new(
        backup_bytes: u64,
        service_name: [u8; 128],
        cadance: Cadance,
        file_extension: [u8; 32],
    ) -> Self {
        assert!(
            is_valid_byte_string(&service_name),
            "Service name '{}' is invalid",
            String::from_utf8_lossy(&service_name)
        );
        assert!(
            is_valid_byte_string(&file_extension),
            "File extension '{}' is invalid",
            String::from_utf8_lossy(&file_extension)
        );

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
            .join(self.serivce_name().to_string())
            .join(self.cadance.as_path())
    }

    #[allow(clippy::missing_safety_doc)]
    /// Creates a new metadata instance.
    /// Does not check service name or file extension.
    pub unsafe fn new_unchecked(
        backup_bytes: u64,
        service_name: [u8; 128],
        cadance: Cadance,
        file_extension: [u8; 32],
    ) -> Self {
        Self {
            backup_bytes,
            service_name,
            cadance,
            file_extension,
        }
    }

    /// The backup's service's name.
    pub fn serivce_name(&self) -> Cow<'_, str> {
        let name_end = self
            .service_name
            .iter()
            .position(|byte| *byte == b'\0')
            .unwrap_or(self.service_name.len());

        String::from_utf8_lossy(&self.service_name[0..name_end])
    }

    /// The backup's file extension.
    pub fn file_extension(&self) -> Cow<'_, str> {
        let name_end = self
            .file_extension
            .iter()
            .position(|byte| *byte == b'\0')
            .unwrap_or(self.file_extension.len());

        String::from_utf8_lossy(&self.file_extension[0..name_end])
    }

    /// Pads or truncates a byte slice to a specified length.
    pub fn pad_string<const L: usize>(bytes: &[u8]) -> [u8; L] {
        let mut output = [b'\0'; L];

        let copy_length = bytes.len().min(L);
        output[..copy_length].copy_from_slice(&bytes[..copy_length]);

        output
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::AnyBitPattern)]
#[allow(missing_docs)]
pub struct MetadataBits {
    backup_bytes: <u64 as CheckedBitPattern>::Bits,
    service_name: <[u8; 128] as CheckedBitPattern>::Bits,
    cadance: <Cadance as CheckedBitPattern>::Bits,
    file_extension: <[u8; 32] as CheckedBitPattern>::Bits,
}

unsafe impl CheckedBitPattern for Metadata {
    type Bits = MetadataBits;

    fn is_valid_bit_pattern(bits: &Self::Bits) -> bool {
        let valid_backup_bytes = u64::is_valid_bit_pattern(&{ bits.backup_bytes });
        let valid_cadance = Cadance::is_valid_bit_pattern(&{ bits.cadance });

        let valid_service_name =
            <[u8; 128] as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.service_name })
                && is_valid_byte_string(&bits.service_name);

        let valid_file_extension =
            <[u8; 32] as CheckedBitPattern>::is_valid_bit_pattern(&{ bits.file_extension })
                && is_valid_byte_string(&bits.file_extension);

        valid_backup_bytes && valid_service_name && valid_cadance && valid_file_extension
    }
}

fn is_valid_byte_string(bytes: &[u8]) -> bool {
    const VALID_CHARACTERS: &[u8; 65] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_\0";

    if bytes.is_empty() {
        return false;
    }

    // All characters must be valid
    if bytes.iter().any(|byte| !VALID_CHARACTERS.contains(byte)) {
        return false;
    }

    // First character must not be nul character
    if bytes[0] == b'\0' {
        return false;
    }

    true
}
