use std::{
    io::{self, Cursor},
    process::Command,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::BackupContents;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerPostgres {
    pub container_name: String,
    pub postgres_username: String,
    pub postgres_database: String,
}

impl DockerPostgres {
    /// Gets the backup from postgres running in docker.
    pub fn get_backup(&self) -> Result<BackupContents, Error> {
        let output = Command::new("docker")
            .args([
                "exec",
                &self.container_name,
                "pg_dump",
                "-U",
                &self.postgres_username,
                "-d",
                &self.postgres_database,
            ])
            .output()
            .map_err(Error::RunCommand)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CommandErrored(error));
        }

        let contents = output.stdout;
        let backup_size = contents.len();
        let contents_reader = Cursor::new(contents);

        Ok(BackupContents {
            backup_size,
            reader: Box::new(contents_reader),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to run command:\n{0}")]
    RunCommand(#[source] io::Error),

    #[error("Command output was error:\n{0}")]
    CommandErrored(String),
}
