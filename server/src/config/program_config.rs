use std::path::PathBuf;

use serde::Deserialize;
use thiserror::Error;

use super::{
    backup_config::BackupConfig,
    certificate::{load_age_key, AgeKeyError},
    RawConfig,
};

#[derive(Clone)]
pub struct ProgramConfig {
    pub backup_path: PathBuf,
    pub age_key: age::x25519::Identity,
    pub service_config: Vec<ServiceConfig>,
}

#[derive(Deserialize, Clone)]
pub struct ServiceConfig {
    pub folder_name: String,
    pub backup_configs: Vec<BackupConfig>,
}

impl TryFrom<&RawConfig> for ProgramConfig {
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        let age_key = load_age_key(&value.age_key_path)?;

        Ok(Self {
            backup_path: value.backup_path.clone(),
            age_key,
            service_config: value.service_config.clone(),
        })
    }
    type Error = ParseProgramConfigError;
}

#[derive(Debug, Error)]
pub enum ParseProgramConfigError {
    #[error("LoadAgeKeyError[br]{0}")]
    LoadAgeKeyError(#[from] AgeKeyError),
}
