use core::str::FromStr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The cadance of a backup.
#[repr(u64)]
#[derive(Hash, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    /// Convert the cadance to big endian bytes.
    pub fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        let value: u64 = unsafe { core::mem::transmute(self) };
        value.to_be_bytes()
    }

    /// Try convert a u64 value to a cadance.
    pub fn try_from_u64(value: u64) -> Option<Self> {
        match value {
            0..=3 => Some(unsafe { core::mem::transmute::<u64, Self>(value) }),
            _ => None,
        }
    }
}

impl FromStr for Cadance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Hourly" => Ok(Self::Hourly),
            "Daily" => Ok(Self::Daily),
            "Weekly" => Ok(Self::Weekly),
            "Monthly" => Ok(Self::Monthly),
            _ => Err(format!("invalid cadance '{s}'")),
        }
    }
}
