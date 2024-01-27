use std::process::Command;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

use super::GetFile;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DockerPostgres {
    pub container_name: String,
    pub postgres_username: String,
    pub postgres_database: String,
}

impl GetFile for DockerPostgres {
    fn get_file(&self) -> anyhow::Result<Vec<u8>> {
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
            .context("Failed to run command")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            bail!(error);
        }

        Ok(output.stdout)
    }
}
