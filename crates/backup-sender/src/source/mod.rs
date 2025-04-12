//! Backup source to retreive backups.
//!

use std::io::BufRead;

use serde::{Deserialize, Serialize};
use shared::Cadance;

mod docker_postgres;

pub use docker_postgres::{DockerPostgres, DockerPostgresError};

use crate::Backup;

/// A source to make a backup of.
pub trait BackupSource {
    /// Error variants.
    type Error;

    /// Reader used to read the backup payload.
    type Reader: BufRead;

    /// Get a backup from the source.
    fn get_backup(&self, cadance: Cadance) -> Result<Backup<Self::Reader>, Self::Error>;
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize, Serialize)]
pub enum Source {
    DockerPostgres(DockerPostgres),
}

impl Source {
    /// The cadance of the backup.
    pub fn cadance(&self) -> &[Cadance] {
        match self {
            Self::DockerPostgres(docker_postgres) => &docker_postgres.cadance,
        }
    }

    pub fn service_name(&self) -> String {
        match self {
            Self::DockerPostgres(docker_postgres) => {
                todo!()
            }
        }
    }
}
