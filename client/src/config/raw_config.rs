use std::path::PathBuf;

use serde::Deserialize;

use crate::backups::backup_types::BackupTypes;

#[derive(Deserialize, Clone)]
pub struct RawConfig {
    pub socket_address: String,
    pub socket_port: u16,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub key_path: PathBuf,
    pub service_config: Vec<BackupTypes>,
}
