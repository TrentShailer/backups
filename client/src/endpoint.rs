use anyhow::Context;
use serde::{Deserialize, Serialize};

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
    fn make_backup(&self, name: &BackupName, max_files: usize, file: &[u8]) -> anyhow::Result<()> {
        match self {
            Endpoint::TlsServer(e) => e
                .make_backup(name, max_files, file)
                .context("Failed to make TlsServer backup")?,
        }
        Ok(())
    }
}

pub trait MakeBackup {
    fn make_backup(&self, name: &BackupName, max_files: usize, file: &[u8]) -> anyhow::Result<()>;
}
