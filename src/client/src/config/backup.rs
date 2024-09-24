use std::time::{Duration, SystemTime};

use thiserror::Error;

use crate::endpoint::{self, Endpoint};
use crate::history::{self, History};
use crate::service::{self, Service};

/// A backup of a service with an endpoint that repeats with a given interval.
pub struct Backup<'a> {
    /// The endpoint to send the backup made.
    pub endpoint: &'a Endpoint,

    /// The service to get the backup from.
    pub service: &'a Service,

    /// The name of the service.
    pub service_name: String,

    /// The name of the endpoint.
    pub endpoint_name: String,

    /// The name of this backup.
    pub backup_name: String,

    /// The interval between backups.
    pub interval: Duration,

    /// The maximum number of files that the endpoint should retain for this backup.
    pub max_files: usize,
}

impl<'a> Backup<'a> {
    /// Returns if this backup is due to be performed.
    pub fn is_backup_due(&self, history: &mut History) -> bool {
        let last_backed_up = history.last_backed_up(self);

        let time_since_last_backed_up = match SystemTime::now().duration_since(last_backed_up) {
            Ok(v) => v,
            Err(_) => Duration::MAX, // if system time changed, we should make a backup
        };

        time_since_last_backed_up >= self.interval
    }

    /// Performs this backup
    pub fn make_backup(&self, history: &mut History) -> Result<(), Error> {
        let mut backup_contents = self.service.get_backup()?;

        self.endpoint.make_backup(
            self.service_name.clone(),
            self.backup_name.clone(),
            self.max_files,
            &mut backup_contents,
        )?;

        history.update(self)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get backup contents:\n{0}")]
    GetBackup(#[from] service::Error),

    #[error("Failed to make backup:\n{0}")]
    MakeBackup(#[from] endpoint::Error),

    #[error("Failed to update history:\n{0}")]
    UpdateHistory(#[from] history::Error),
}
