use serde::{Deserialize, Serialize};

use crate::scheduler_config::{SchedulerBackup, SchedulerConfig, SchedulerService};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    pub socket_address: String,
    pub socket_port: u16,
    pub container_name: String,
    pub postgres_username: String,
    pub postgres_database: String,
    pub service_name: String,
    pub backup_name: String,
    pub max_files: usize,
}

impl BackupConfig {
    pub fn from_scheduler(
        config: &SchedulerConfig,
        service: &SchedulerService,
        backup: &SchedulerBackup,
    ) -> Self {
        Self {
            socket_address: config.socket_address.clone(),
            socket_port: config.socket_port,
            container_name: service.container_name.clone(),
            postgres_username: service.postgres_username.clone(),
            postgres_database: service.postgres_database.clone(),
            service_name: service.service_name.clone(),
            backup_name: backup.backup_name.clone(),
            max_files: backup.max_files,
        }
    }
}
