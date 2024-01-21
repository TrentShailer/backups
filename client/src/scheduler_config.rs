use serde::Deserialize;

use crate::{endpoint::Endpoint, service::ServiceConfig};

#[derive(Debug, Deserialize)]
pub struct SchedulerConfig {
    pub endpoints: Vec<Endpoint>,
    pub services: Vec<SchedulerService>,
}

#[derive(Debug, Deserialize)]
pub struct SchedulerService {
    pub config: ServiceConfig,
    pub service_name: String,
    pub backups: Vec<SchedulerBackup>,
}

#[derive(Debug, Deserialize)]
pub struct SchedulerBackup {
    pub backup_name: String,
    pub interval: u64,
    pub max_files: usize,
}

pub struct BackupName {
    pub endpoint_name: String,
    pub service_name: String,
    pub backup_name: String,
}

impl BackupName {
    pub fn new(endpoint_name: &str, service_name: &str, backup_name: &str) -> Self {
        Self {
            endpoint_name: endpoint_name.to_string(),
            service_name: service_name.to_string(),
            backup_name: backup_name.to_string(),
        }
    }
}

impl ToString for BackupName {
    fn to_string(&self) -> String {
        format!(
            "{}/{}/{}",
            self.endpoint_name, self.service_name, self.backup_name
        )
    }
}
