use crate::backups::backup_types::BackupTypes;

use super::raw_config::RawConfig;

#[derive(Clone)]
pub struct ProgramConfig {
    pub service_config: Vec<BackupTypes>,
}

impl From<&RawConfig> for ProgramConfig {
    fn from(value: &RawConfig) -> Self {
        Self {
            service_config: value.service_config.clone(),
        }
    }
}
