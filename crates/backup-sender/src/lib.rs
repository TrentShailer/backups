//! # backup-sender
//!

use std::io::BufRead;

use shared::Metadata;

pub mod config;
pub mod context;
pub mod endpoint;
pub mod history;
pub mod source;

/// A backup.
pub struct Backup<Reader: BufRead> {
    /// The backup's metadata.
    pub metadata: Metadata,
    /// The reader to read the backup payload.
    pub reader: Reader,
}
