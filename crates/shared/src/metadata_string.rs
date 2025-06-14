use serde::{Deserialize, Serialize, de};
use thiserror::Error;

use crate::Failure;

/// A stack allocated string that only accepts `[a-zA-Z0-9_\-]`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MetadataString<const L: usize> {
    bytes: [u8; L],
}

impl<const L: usize> MetadataString<L> {
    /// Create a new metadata string from bytes without checking if it is valid.
    ///
    /// # Safety
    /// * `bytes` must be a valid representation of MetadataString.
    pub unsafe fn new_unchecked(bytes: [u8; L]) -> Self {
        Self { bytes }
    }

    /// Converts the metadata string to a string.
    pub fn as_string(&self) -> String {
        let end = self
            .bytes
            .iter()
            .position(|byte| *byte == b'\0')
            .unwrap_or(L);

        String::from_utf8(self.bytes[..end].to_vec())
            .or_log_and_panic("Metadata string MUST be valid UTF-8")
    }

    /// Validate if a slice of bytes make a valid `MetadataString`.
    pub fn validate_bytes(bytes: &[u8]) -> Result<(), MetadataStringError> {
        const VALID_CHARACTERS: &[u8; 65] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_\0";

        if bytes.is_empty() {
            return Err(MetadataStringError::Empty);
        }

        // Check length
        if bytes.len() > L {
            return Err(MetadataStringError::TooLong(bytes.len(), L));
        }

        // All characters must be valid
        if let Some((index, byte)) = bytes
            .iter()
            .enumerate()
            .find(|(_, byte)| !VALID_CHARACTERS.contains(byte))
        {
            return Err(MetadataStringError::Invalid(
                index,
                *byte,
                char::from(*byte),
            ));
        }

        // First character must not be null character
        if bytes[0] == b'\0' {
            return Err(MetadataStringError::Invalid(0, b'\0', '\0'));
        }

        Ok(())
    }

    /// Returns the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; L] {
        &self.bytes
    }
}

impl<const L: usize> TryFrom<&[u8]> for MetadataString<L> {
    type Error = MetadataStringError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::validate_bytes(bytes)?;

        // Create byte string
        let mut valid_bytes = [b'\0'; L];
        let copy_length = bytes.len().min(L);
        valid_bytes[..copy_length].copy_from_slice(&bytes[..copy_length]);

        Ok(Self { bytes: valid_bytes })
    }
}

impl<const L: usize> TryFrom<String> for MetadataString<L> {
    type Error = MetadataStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl<const L: usize> TryFrom<&str> for MetadataString<L> {
    type Error = MetadataStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl<const L: usize> Default for MetadataString<L> {
    fn default() -> Self {
        let mut bytes = [0u8; L];
        bytes[0] = b'_';
        Self { bytes }
    }
}

impl<const L: usize> core::fmt::Debug for MetadataString<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MetadataString")
            .field("string", &self.as_string())
            .field("string_bytes", &self.bytes)
            .finish()
    }
}

impl<const L: usize> core::fmt::Display for MetadataString<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}
impl<'de, const L: usize> Deserialize<'de> for MetadataString<L> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = Deserialize::deserialize(deserializer)?;
        Self::try_from(string).map_err(de::Error::custom)
    }
}

impl<const L: usize> Serialize for MetadataString<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = self.as_string();
        serializer.serialize_str(&string)
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MetadataStringError {
    /// `length, limit`
    #[error("Input was too long {0} > {0}")]
    TooLong(usize, usize),

    #[error("Input was empty")]
    Empty,

    /// `index, byte, char`
    #[error("Invalid byte as index {0}: '{1}' ('{2}'), may only contain [a-zA-Z0-9_\\-]")]
    Invalid(usize, u8, char),
}
