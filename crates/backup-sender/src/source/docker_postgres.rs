use core::num::TryFromIntError;
use std::{
    io::{self, Cursor},
    process::Command,
};

use serde::{Deserialize, Serialize};
use shared::{Cadence, Metadata, MetadataString};
use thiserror::Error;

use super::{Backup, BackupSource};

/// Make a backup from a postgres docker container.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DockerPostgres {
    /// The name of the container.
    pub container_name: String,
    /// The postgres username.
    pub postgres_username: String,
    /// The postgres database.
    pub postgres_database: String,

    /// The service name.
    pub service_name: MetadataString<128>,

    /// The file extension.
    pub file_extension: MetadataString<32>,

    /// The cadences to backup this source.
    pub cadence: Vec<Cadence>,
}

impl BackupSource for DockerPostgres {
    type Error = DockerPostgresError;

    fn get_backup(&self, cadence: Cadence) -> Result<Backup, Self::Error> {
        let output = Command::new("docker")
            .args([
                "exec",
                &self.container_name,
                "pg_dump",
                "-U",
                &self.postgres_username,
                "-d",
                &self.postgres_database,
                "-a",
            ])
            .output()
            .map_err(DockerPostgresError::RunCommand)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(DockerPostgresError::CommandErrored(error));
        }

        let contents = output.stdout;
        let backup_size = u64::try_from(contents.len())?;

        let metadata = Metadata::new(backup_size, self.service_name, cadence, self.file_extension);

        Ok(Backup {
            metadata,
            reader: Box::new(Cursor::new(contents)),
        })
    }

    fn cadence(&self) -> &[Cadence] {
        &self.cadence
    }

    fn service_name(&self) -> String {
        self.service_name.as_string()
    }

    fn cleanup(&self, _metadata: Metadata) {}
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum DockerPostgresError {
    #[error("Failed to run command:\n{0}")]
    RunCommand(#[source] io::Error),

    #[error("Command output was error:\n{0}")]
    CommandErrored(String),

    #[error("Backup was larger than u64::MAX: {0}")]
    BackupTooLarge(#[from] TryFromIntError),
}
