pub mod tls_server;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tls_server::TlsServer;

use crate::service::BackupContents;

/// Endpoints to receive the backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Endpoint {
    TlsServer(TlsServer),
}

impl Endpoint {
    /// Make the backup to the specified endpoint.
    pub fn make_backup(
        &self,
        service_name: String,
        backup_name: String,
        max_files: usize,
        backup_contents: &mut BackupContents,
    ) -> Result<(), Error> {
        match self {
            Endpoint::TlsServer(tls_server) => {
                tls_server.make_backup(service_name, backup_name, max_files, backup_contents)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to make TLS endpoint backup:\n{0}")]
    Tls(#[from] tls_server::Error),
}
