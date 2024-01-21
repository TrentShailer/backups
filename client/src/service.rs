use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::docker_postgres::DockerPostgres;

mod docker_postgres;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceConfig {
    DockerPostgres(DockerPostgres),
}

impl GetFile for ServiceConfig {
    type Error = GetFileError;

    async fn get_file(&self) -> Result<Vec<u8>, Self::Error> {
        let file = match self {
            ServiceConfig::DockerPostgres(c) => c.get_file().await?,
        };
        Ok(file)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GetFileError {
    #[error(transparent)]
    DockerPostgres(#[from] docker_postgres::GetFileError),
}

pub trait GetFile {
    type Error;
    async fn get_file(&self) -> Result<Vec<u8>, Self::Error>;
}
