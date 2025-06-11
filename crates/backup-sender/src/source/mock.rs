use core::num::TryFromIntError;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use shared::{Cadence, Metadata, MetadataString};

use super::{Backup, BackupSource};

/// Mock a backup source.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Mock {
    /// The service name.
    pub service_name: MetadataString<128>,

    /// The file extension.
    pub file_extension: MetadataString<32>,

    /// The cadences to backup this source.
    pub cadence: Vec<Cadence>,
}

impl BackupSource for Mock {
    type Error = TryFromIntError;

    fn get_backup(&self, cadence: Cadence) -> Result<Backup, Self::Error> {
        let data = vec![0u8; 512];
        let backup_size = u64::try_from(data.len())?;

        let metadata = Metadata::new(backup_size, self.service_name, cadence, self.file_extension);

        Ok(Backup {
            metadata,
            reader: Box::new(Cursor::new(data)),
        })
    }

    fn cadence(&self) -> &[Cadence] {
        &self.cadence
    }

    fn service_name(&self) -> String {
        self.service_name.as_string()
    }

    fn cleanup(&self, _metadata: Metadata) {}
}
