use thiserror::Error;
use tracing::debug;

use crate::{backups::backup_types::BackupTypes, config::certificate::load_recipiant};

use super::{certificate::LoadRecipiantError, raw_config::RawConfig};

#[derive(Clone)]
pub struct ProgramConfig {
    pub recipiant: age::x25519::Recipient,
    pub service_config: Vec<BackupTypes>,
}

impl TryFrom<&RawConfig> for ProgramConfig {
    #[tracing::instrument(level = "trace", skip_all, err)]
    fn try_from(value: &RawConfig) -> Result<Self, Self::Error> {
        debug!("Started parsing program config");

        let recipiant = load_recipiant(&value.recipiant_path)?;
        debug!("Loaded recipiant");

        Ok(Self {
            recipiant,
            service_config: value.service_config.clone(),
        })
    }

    type Error = ParseProgramConfigError;
}

#[derive(Debug, Error)]
pub enum ParseProgramConfigError {
    #[error("LoadRecipiantError -> {0}")]
    LoadRecipiantError(#[from] LoadRecipiantError),
}
