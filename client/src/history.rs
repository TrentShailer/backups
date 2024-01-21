mod history_items;

use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    time::SystemTime,
};

use log::error;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;

use crate::scheduler_config::BackupName;

use self::history_items::EndpointHistory;

const HISTORY_PATH: &str = "./history.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub endpoints: Vec<EndpointHistory>,
}

impl History {
    pub fn init() -> Result<Self, InitError> {
        let path = PathBuf::from(HISTORY_PATH);
        if path.exists() {
            let mut file = File::open(path).map_err(InitError::OpenFile)?;
            let mut contents = String::new();
            File::read_to_string(&mut file, &mut contents).map_err(InitError::ReadFile)?;

            let history: Self = toml::from_str(&contents)?;

            Ok(history)
        } else {
            Ok(Self { endpoints: vec![] })
        }
    }

    pub fn last_backed_up(&self, name: &BackupName) -> SystemTime {
        for endpoint in self.endpoints.iter() {
            if let Some(v) = endpoint.find(name) {
                return v;
            }
        }
        return SystemTime::UNIX_EPOCH;
    }

    pub async fn update(&mut self, name: &BackupName) -> Result<(), SaveError> {
        let mut found = false;
        for endpoint in self.endpoints.iter_mut() {
            if endpoint.endpoint_name == name.endpoint_name {
                endpoint.update(name);
                found = true;
                break;
            }
        }

        if found == false {
            let endpoint = EndpointHistory::create(name);
            self.endpoints.push(endpoint);
        }

        Ok(self.save().await?)
    }

    async fn save(&self) -> Result<(), SaveError> {
        let contents = toml::to_string(self)?;
        fs::write(PathBuf::from(HISTORY_PATH), contents).await?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("OpenFileError:\n{0}")]
    OpenFile(#[source] io::Error),
    #[error("ReadFileError:\n{0}")]
    ReadFile(#[source] io::Error),
    #[error("DeserializeError:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("WriteError:\n{0}")]
    Write(#[from] io::Error),
    #[error("SerializeError:\n{0}")]
    Serialize(#[from] toml::ser::Error),
}
