use core::num::TryFromIntError;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use shared::{Cadance, Metadata, MetadataString};

use super::{Backup, BackupSource};

/// Mock a backup source.
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Mock {
    /// The service name.
    pub service_name: MetadataString<128>,

    /// The file extension.
    pub file_extension: MetadataString<32>,

    /// The cadances to backup this source.
    pub cadance: Vec<Cadance>,
}

impl BackupSource for Mock {
    type Error = TryFromIntError;
    type Reader = Cursor<Vec<u8>>;

    fn get_backup(&self, cadance: Cadance) -> Result<Backup<Self::Reader>, Self::Error> {
        let data = vec![0u8; 512];
        let backup_size = u64::try_from(data.len())?;

        let metadata = Metadata::new(backup_size, self.service_name, cadance, self.file_extension);

        Ok(Backup {
            metadata,
            reader: Cursor::new(data),
        })
    }

    fn cadance(&self) -> &[Cadance] {
        &self.cadance
    }

    fn service_name(&self) -> String {
        self.service_name.as_string()
    }
}
