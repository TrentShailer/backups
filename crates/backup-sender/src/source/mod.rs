//! Backup source to retreive backups.
//!

use core::fmt::{Debug, Display};

use folder_tar::FolderTar;
use mock::Mock;
use serde::{Deserialize, Serialize};
use shared::Cadance;

mod docker_postgres;
mod folder_tar;
mod mock;

pub use docker_postgres::{DockerPostgres, DockerPostgresError};

use crate::Backup;

/// A source to make a backup of.
pub trait BackupSource: Debug + Serialize + for<'a> Deserialize<'a> {
    /// Error variants.
    type Error: Display;

    /// Get a backup from the source.
    fn get_backup(&self, cadance: Cadance) -> Result<Backup, Self::Error>;

    /// The cadances for the service.
    fn cadance(&self) -> &[Cadance];

    /// The service name being backed up.
    fn service_name(&self) -> String;
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize, Serialize)]
pub enum Source {
    DockerPostgres(DockerPostgres),
    FolderTar(FolderTar),
    Mock(Mock),
}

impl Source {
    /// The cadance of the backup.
    pub fn cadance(&self) -> &[Cadance] {
        match self {
            Self::DockerPostgres(docker_postgres) => &docker_postgres.cadance,
            Self::FolderTar(folder_tar) => &folder_tar.cadance,
            Self::Mock(mock) => &mock.cadance,
        }
    }

    /// The service name of the backup
    pub fn service_name(&self) -> String {
        match self {
            Self::DockerPostgres(docker_postgres) => docker_postgres.service_name.as_string(),
            Self::FolderTar(folder_tar) => folder_tar.service_name.as_string(),
            Self::Mock(mock) => mock.service_name.as_string(),
        }
    }
}
