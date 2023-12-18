use std::path::PathBuf;

use serde::Deserialize;

use super::{backup_config::BackupConfig, RawConfig};

#[derive(Clone)]
pub struct ProgramConfig {
    pub backup_path: PathBuf,
    pub service_config: Vec<ServiceConfig>,
}

#[derive(Deserialize, Clone)]
pub struct ServiceConfig {
    pub folder_name: String,
    pub backup_configs: Vec<BackupConfig>,
}

impl From<&RawConfig> for ProgramConfig {
    fn from(value: &RawConfig) -> Self {
        Self {
            backup_path: value.backup_path.clone(),
            service_config: value.service_config.clone(),
        }
    }
}
