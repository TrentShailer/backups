use anyhow::Context;
use serde::{Deserialize, Serialize};

use self::docker_postgres::DockerPostgres;

mod docker_postgres;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Service {
    DockerPostgres(DockerPostgres),
}

impl GetFile for Service {
    fn get_file(&self) -> anyhow::Result<Vec<u8>> {
        let file = match self {
            Service::DockerPostgres(c) => {
                c.get_file().context("Failed to get DockerPostgres file")?
            }
        };
        Ok(file)
    }
}

pub trait GetFile {
    fn get_file(&self) -> anyhow::Result<Vec<u8>>;
}
