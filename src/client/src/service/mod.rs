use std::io::Read;

use dummy::Dummy;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::docker_postgres::DockerPostgres;

mod docker_postgres;
mod dummy;

/// The contents of a backup.
pub struct BackupContents {
    /// Size of backup in bytes.
    pub backup_size: u64,

    /// Reader that yields the contents of the backup.
    pub reader: Box<dyn Read>,
}

/// Service to source the backup file from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Service {
    DockerPostgres(DockerPostgres),
    Dummy(Dummy),
}

impl Service {
    pub fn get_backup(&self) -> Result<BackupContents, Error> {
        let backup_contents = match self {
            Service::DockerPostgres(docker_postgres) => docker_postgres.get_backup()?,
            Service::Dummy(dummy) => dummy.get_backup()?,
        };

        Ok(backup_contents)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get backup from docker postgres:\n{0}")]
    DockerPostgres(#[from] docker_postgres::Error),

    #[error("Failed to get backup from dummy:\n{0}")]
    Dummy(#[from] dummy::Error),
}
