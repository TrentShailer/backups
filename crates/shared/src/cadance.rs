use std::path::PathBuf;

use bytemuck::{CheckedBitPattern, NoUninit};
use serde::{Deserialize, Serialize};

/// The cadance of a backup.
#[repr(u64)]
#[derive(
    Hash, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, CheckedBitPattern, NoUninit,
)]
pub enum Cadance {
    /// A backup should be sent every hour.
    Hourly = 0,

    /// A backup should be sent every day.
    Daily = 1,

    /// A backup should be sent every week,
    Weekly = 2,

    /// A backup should be sent every month.
    Monthly = 3,
}

impl Cadance {
    /// Interprets the Cadance as a pathbuf segment.
    pub fn as_path(&self) -> PathBuf {
        match self {
            Self::Hourly => "hourly".into(),
            Self::Daily => "daily".into(),
            Self::Weekly => "weekly".into(),
            Self::Monthly => "monthly".into(),
        }
    }
}
