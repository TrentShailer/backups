use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;

use crate::endpoint::{Endpoint, MakeBackup};
use crate::history::History;
use crate::scheduler_config::BackupName;
use crate::service::{GetFile, Service};

pub struct Backup<'a> {
    pub endpoint: &'a Endpoint,
    pub service: &'a Service,
    pub name: BackupName,
    pub interval: Duration,
    pub max_files: usize,
}

impl<'a> Backup<'a> {
    pub fn new(
        endpoint: &'a Endpoint,
        service: &'a Service,
        name: BackupName,
        interval: Duration,
        max_files: usize,
    ) -> Self {
        Self {
            endpoint,
            service,
            name,
            interval,
            max_files,
        }
    }

    pub fn maybe_make_backup(&self, history: &mut History) -> anyhow::Result<()> {
        log::debug!("Maybe making backup for {}", self.name.to_string());
        let last_backed_up = history.last_backed_up(&self.name);

        let time_since_last_backed_up = match SystemTime::now().duration_since(last_backed_up) {
            Ok(v) => v,
            Err(_) => Duration::MAX, // if system time changed, we should make a backup
        };

        if time_since_last_backed_up < self.interval {
            log::debug!(
                "{}: Not ready for backup. time since last backup: {}",
                self.name.to_string(),
                time_since_last_backed_up.as_secs_f64(),
            );
            return Ok(());
        }

        log::debug!("{}: Starting getting file", self.name.to_string());
        let file = Arc::new(self.service.get_file().context("Failed to get file")?);
        log::debug!("{}: Got file", self.name.to_string());
        let file = Arc::clone(&file);

        log::debug!("{}: Starting sending backup", self.name.to_string());
        self.endpoint
            .make_backup(&self.name, self.max_files, &file)
            .context("Failed to make backup")?;
        log::debug!("{}: Sent backup", self.name.to_string());

        log::debug!("{}: Starting updaing history", self.name.to_string());
        history
            .update(&self.name)
            .context("Failed to update history")?;
        log::debug!("{}: Updated history", self.name.to_string());

        Ok(())
    }
}
