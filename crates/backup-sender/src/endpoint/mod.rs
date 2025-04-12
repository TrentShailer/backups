//! Endpoints for the backups
//!

pub use backup_receiver::{BackupReceiverError, BackupRecevier};

use std::io::BufRead;

use serde::{Deserialize, Serialize};

use crate::Backup;

mod backup_receiver;

/// An endpoint to send backups to.
pub trait BackupEndpoint {
    /// The error variants.
    type Error;

    /// Send a backup to the endpoint.
    fn send_backup<Reader: BufRead>(&mut self, content: Backup<Reader>) -> Result<(), Self::Error>;
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize, Serialize)]
pub enum Endpoint {
    BackupReceiver(BackupRecevier),
}
