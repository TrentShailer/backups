use thiserror::Error;

use crate::{backups::backup_types::BackupTypes, config::certificate::load_recipiant};

use super::{certificate::LoadRecipiantError, raw_config::RawConfig};

#[derive(Clone)]
pub struct ProgramConfig {
    pub recipiant: age::x25519::Recipient,
    pub service_config: Vec<BackupTypes>,
}

impl TryFrom<&RawConfig> for ProgramConfig {
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        let recipiant = load_recipiant(&value.recipiant_path)?;

        Ok(Self {
            recipiant,
            service_config: value.service_config.clone(),
        })
    }

    type Error = ParseProgramConfigError;
}

#[derive(Debug, Error)]
pub enum ParseProgramConfigError {
    #[error("LoadRecipiantError\n{0}")]
    LoadRecipiantError(#[from] LoadRecipiantError),
}
