use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{io, process::Command};

use super::GetFile;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerPostgres {
    pub container_name: String,
    pub postgres_username: String,
    pub postgres_database: String,
}

impl GetFile for DockerPostgres {
    type Error = GetFileError;

    async fn get_file(&self) -> Result<Vec<u8>, Self::Error> {
        let output = Command::new("docker")
            .args(&[
                "exec",
                &self.container_name,
                "pg_dump",
                "-U",
                &self.postgres_username,
                "-d",
                &self.postgres_database,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Self::Error::CommandResult(error));
        }

        Ok(output.stdout)
    }
}

#[derive(Debug, Error)]
pub enum GetFileError {
    #[error("CommandError:\n{0}")]
    Command(#[from] io::Error),
    #[error("CommandResultError:\n{0}")]
    CommandResult(String),
}
