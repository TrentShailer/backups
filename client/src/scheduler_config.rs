use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SchedulerConfig {
    pub socket_address: String,
    pub socket_port: u16,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub key_path: PathBuf,
    pub services: Vec<SchedulerService>,
}

#[derive(Debug, Deserialize)]
pub struct SchedulerService {
    pub container_name: String,
    pub postgres_username: String,
    pub postgres_database: String,
    pub service_name: String,
    pub backups: Vec<SchedulerBackup>,
}

#[derive(Debug, Deserialize)]
pub struct SchedulerBackup {
    pub backup_name: String,
    pub interval: u64,
    pub max_files: usize,
}
