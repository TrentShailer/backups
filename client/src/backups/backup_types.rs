mod docker_postgres;

use std::time::Duration;

use serde::Deserialize;

use self::docker_postgres::DockerPostgresBackupConfig;

#[derive(Deserialize, Clone)]
pub enum BackupTypes {
    DockerPostgres { config: DockerPostgresBackupConfig },
}

#[serde_with::serde_as]
#[derive(Deserialize, Clone)]
pub struct BackupConfig {
    pub folder_name: String,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    pub backup_interval: Duration,
}
