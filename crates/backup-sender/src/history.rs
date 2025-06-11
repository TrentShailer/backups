//! Backup history
//!

use core::time::Duration;
use std::{collections::HashMap, fs, io, path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize, de};
use shared::Cadence;
use thiserror::Error;
use tracing::warn;

const HISTORY_FILE: &str = "history.json";

/// History key
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct HistoryKey {
    /// Service name.
    pub service: String,
    /// Backup cadence.
    pub cadence: Cadence,
}

impl HistoryKey {
    /// Create a new history key.
    pub fn new(service: String, cadence: Cadence) -> Self {
        Self { service, cadence }
    }
}

impl<'de> Deserialize<'de> for HistoryKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string: String = Deserialize::deserialize(deserializer)?;
        let parts: Vec<_> = string.split("::").collect();
        if parts.len() != 2 {
            return Err(de::Error::custom("invalid key format"));
        }

        let service = parts[0].to_string();
        let cadence = parts[1].parse().map_err(de::Error::custom)?;

        Ok(Self { service, cadence })
    }
}

impl Serialize for HistoryKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let service = &self.service;
        let cadence = self.cadence;
        serializer.serialize_str(&format!("{service}::{cadence:?}"))
    }
}

/// The history of given cadences of a given service.
#[derive(Debug, Deserialize, Serialize)]
pub struct History {
    /// The history
    pub history: HashMap<HistoryKey, SystemTime>,
}

impl History {
    /// Create a new instance of the history.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    /// Tries to load the history from a json file.
    pub fn load_or_create_file() -> Result<Self, LoadHistoryError> {
        if !PathBuf::from(HISTORY_FILE).exists() {
            let history = Self::new();
            history.save()?;
            return Ok(history);
        }

        let contents =
            fs::read_to_string(PathBuf::from(HISTORY_FILE)).map_err(LoadHistoryError::ReadFile)?;
        let config = serde_json::from_str(&contents)?;

        Ok(config)
    }

    /// Returns if a given service's cadence needs to be backed up.
    pub fn needs_backup(&self, service_name: String, cadence: Cadence) -> bool {
        let last_backed_up = match self.history.get(&HistoryKey::new(service_name, cadence)) {
            Some(backed_up) => backed_up,
            None => return true,
        };

        let elapsed = match SystemTime::now().duration_since(*last_backed_up) {
            Ok(elapsed) => elapsed,
            Err(error) => {
                warn!("System time may have changed: {error}");
                return true;
            }
        };

        match cadence {
            Cadence::Hourly => elapsed >= Duration::from_secs(60 * 60),
            Cadence::Daily => elapsed >= Duration::from_secs(60 * 60 * 24),
            Cadence::Weekly => elapsed >= Duration::from_secs(60 * 60 * 24 * 7),
            Cadence::Monthly => elapsed >= Duration::from_secs(60 * 60 * 24 * 30),
        }
    }

    /// Update the history for a given cadence and save.
    pub fn update(
        &mut self,
        service_name: String,
        cadence: Cadence,
    ) -> Result<(), SaveHistoryError> {
        self.history
            .insert(HistoryKey::new(service_name, cadence), SystemTime::now());

        self.save()?;

        Ok(())
    }

    /// Save the current history.
    pub fn save(&self) -> Result<(), SaveHistoryError> {
        let contents = serde_json::to_string(self)?;
        fs::write(PathBuf::from(HISTORY_FILE), contents).map_err(SaveHistoryError::WriteFile)?;
        Ok(())
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum LoadHistoryError {
    #[error("Failed to deserialize history: {0}")]
    Deserialize(#[from] serde_json::error::Error),

    #[error("Failed to create history file: {0}")]
    CreateFile(#[source] io::Error),

    #[error("Failed to read history: {0}")]
    ReadFile(#[source] io::Error),

    #[error("Failed to create new history file: {0}")]
    CreateHistory(#[from] SaveHistoryError),
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum SaveHistoryError {
    #[error("Failed to serialize history: {0}")]
    Serialize(#[from] serde_json::error::Error),

    #[error("Failed to write history file: {0}")]
    WriteFile(#[source] io::Error),
}
