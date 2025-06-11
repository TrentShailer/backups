//! # Shared
//! The shared components between the backup receiver and sender.
//!

#![warn(missing_docs)]

mod cadence;
mod certificates;
mod endian;
mod failure;
mod logger;
mod metadata;
mod metadata_string;
mod response;
#[cfg(feature = "test")]
pub mod test;

pub use cadence::Cadence;
pub use certificates::{CertificateError, Certificates};
pub use endian::Endian;
pub use failure::Failure;
pub use logger::{LoggerError, init_logger};
pub use metadata::{Metadata, MetadataError};
pub use metadata_string::{MetadataString, MetadataStringError};
pub use response::Response;
