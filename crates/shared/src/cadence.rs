use core::str::FromStr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The cadence of a backup.
#[repr(u64)]
#[derive(Hash, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Cadence {
    /// A backup should be sent every hour.
    Hourly = 0,

    /// A backup should be sent every day.
    Daily = 1,

    /// A backup should be sent every week,
    Weekly = 2,

    /// A backup should be sent every month.
    Monthly = 3,
}

impl Cadence {
    /// Interprets the Cadence as a `pathbuf` segment.
    pub fn as_path(&self) -> PathBuf {
        match self {
            Self::Hourly => "hourly".into(),
            Self::Daily => "daily".into(),
            Self::Weekly => "weekly".into(),
            Self::Monthly => "monthly".into(),
        }
    }

    /// Try convert a `u64` value to a cadence.
    pub fn try_from_u64(value: u64) -> Option<Self> {
        match value {
            0..=3 => Some(unsafe { core::mem::transmute::<u64, Self>(value) }),
            _ => None,
        }
    }

    /// Verify if a number if a valid instance of `Cadence`.
    pub fn is_valid(value: u64) -> bool {
        matches!(value, 0..=3)
    }
}

impl FromStr for Cadence {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Hourly" => Ok(Self::Hourly),
            "Daily" => Ok(Self::Daily),
            "Weekly" => Ok(Self::Weekly),
            "Monthly" => Ok(Self::Monthly),
            _ => Err(format!("invalid cadence '{s}'")),
        }
    }
}
