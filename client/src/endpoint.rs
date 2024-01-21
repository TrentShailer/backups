use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::scheduler_config::BackupName;

use self::tls_server::TlsServer;

pub mod tls_server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Endpoint {
    TlsServer(TlsServer),
}

impl Endpoint {
    pub fn name(&self) -> &str {
        match self {
            Endpoint::TlsServer(_) => "TlsServer",
        }
    }
}

impl MakeBackup for Endpoint {
    type Error = MakeBackupError;

    async fn make_backup(
        &self,
        name: &BackupName,
        max_files: usize,
        file: &[u8],
    ) -> Result<(), Self::Error> {
        match self {
            Endpoint::TlsServer(e) => e.make_backup(name, max_files, file).await?,
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum MakeBackupError {
    #[error(transparent)]
    TlsServer(#[from] tls_server::MakeBackupError),
}

pub trait MakeBackup {
    type Error;
    async fn make_backup(
        &self,
        name: &BackupName,
        max_files: usize,
        file: &[u8],
    ) -> Result<(), Self::Error>;
}
