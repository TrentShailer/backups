use std::io::Cursor;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::BackupContents;

/// A dummy service that yields a backup with 2KiB of data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dummy {}

impl Dummy {
    /// Gets the backup with dummy data.
    pub fn get_backup(&self) -> Result<BackupContents, Error> {
        let backup_size = 1024 * 2;

        let data = [b't'; 1024 * 2];
        let reader = Cursor::new(data);

        Ok(BackupContents {
            backup_size,
            reader: Box::new(reader),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {}
