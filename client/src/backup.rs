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
        let last_backed_up = history.last_backed_up(&self.name);

        let time_since_last_backed_up = match SystemTime::now().duration_since(last_backed_up) {
            Ok(v) => v,
            Err(_) => Duration::MAX, // if system time changed, we should make a backup
        };

        if time_since_last_backed_up < self.interval {
            return Ok(());
        }

        let file = Arc::new(self.service.get_file().context("Failed to get file")?);

        let file = Arc::clone(&file);

        self.endpoint
            .make_backup(&self.name, self.max_files, &file)
            .context("Failed to make backup")?;

        history
            .update(&self.name)
            .context("Failed to update history")?;

        Ok(())
    }
}
