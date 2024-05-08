use error_trace::{ErrorTrace, ResultExt};
use serde::{Deserialize, Serialize};

use self::docker_postgres::DockerPostgres;

mod docker_postgres;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Service {
    DockerPostgres(DockerPostgres),
}

impl GetFile for Service {
    fn get_file(&self) -> Result<Vec<u8>, ErrorTrace> {
        let file = match self {
            Service::DockerPostgres(c) => c.get_file().context("Get DockerPostgres file")?,
        };
        Ok(file)
    }
}

pub trait GetFile {
    fn get_file(&self) -> Result<Vec<u8>, ErrorTrace>;
}
