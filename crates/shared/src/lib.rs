//! # Shared
//! The shared components between the backup receiver and sender.
//!

#![warn(missing_docs)]

mod cadance;
mod certificates;
mod logger;
mod metadata;
mod response;
#[cfg(feature = "test")]
pub mod test;

pub use cadance::Cadance;
pub use certificates::{CertificateError, Certificates};
pub use logger::{LoggerError, init_logger};
pub use metadata::Metadata;
pub use response::Response;
