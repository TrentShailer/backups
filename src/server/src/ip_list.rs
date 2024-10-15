use std::{
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const IP_LIST_PATH: &str = "./ip_list.toml";

/// Persistant data structure that tracks blocked and trusted IPs.
#[derive(Debug, Serialize, Deserialize)]
pub struct IpList {
    /// List of blocked ips.
    blocked_ips: Vec<IpAddr>,

    /// List of trusted IPs
    trusted_ips: Vec<IpAddr>,

    /// Path to ip list file
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl IpList {
    /// Tries to load an existing ip list, or create a new one if none exist.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        // If file doesn't exist, create new.
        if !path.as_ref().exists() {
            let ip_list = Self {
                blocked_ips: vec![],
                trusted_ips: vec![],
                file_path: Some(path.as_ref().to_path_buf()),
            };

            ip_list.save()?;
            return Ok(ip_list);
        }

        // File exists, read and deserialze
        let contents = fs::read_to_string(path).map_err(Error::Read)?;
        let ip_list = toml::from_str(&contents)?;

        Ok(ip_list)
    }

    #[allow(dead_code)]
    /// Creates a new unbacked ip list, does not support saving or loading.
    pub fn new_unbacked() -> Self {
        Self {
            blocked_ips: vec![],
            trusted_ips: vec![],
            file_path: None,
        }
    }

    #[allow(dead_code)]
    /// Returns a reference to the the blocked ips.
    pub fn get_blocked_ips(&self) -> &[IpAddr] {
        &self.blocked_ips
    }

    #[allow(dead_code)]
    /// Returns a reference to the the trusted ips.
    pub fn get_trusted_ips(&self) -> &[IpAddr] {
        &self.trusted_ips
    }

    /// Adds the ip to the block list if it isn't trusted.
    /// Then saves the changes.
    pub fn block_untrusted(&mut self, ip: IpAddr) -> Result<(), Error> {
        if self.is_blocked(&ip) || self.is_trusted(&ip) {
            return Ok(());
        }

        self.blocked_ips.push(ip);
        self.save()?;

        Ok(())
    }

    /// Adds the ip to the trust list if it isn't isn't blocked.
    /// Then saves the changes.
    pub fn trust_unblocked(&mut self, ip: IpAddr) -> Result<(), Error> {
        if self.is_blocked(&ip) || self.is_trusted(&ip) {
            return Ok(());
        }

        self.trusted_ips.push(ip);
        self.save()?;

        Ok(())
    }

    /// Returns if the ip is in the block list.
    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        self.blocked_ips.contains(ip)
    }

    /// Returns if the ip is in the trust list.
    pub fn is_trusted(&self, ip: &IpAddr) -> bool {
        self.trusted_ips.contains(ip)
    }

    /// Serializes and saves the lists to the file.
    fn save(&self) -> Result<(), Error> {
        let Some(file_path) = self.file_path.as_ref() else {
            return Ok(());
        };

        let contents = toml::to_string_pretty(self)?;
        fs::write(file_path, contents).map_err(Error::Write)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to read to file:\n{0}")]
    Read(#[source] std::io::Error),

    #[error("Failed to write to file:\n{0}")]
    Write(#[source] std::io::Error),

    #[error("Failed to serialize:\n{0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("Failed to deserialize:\n{0}")]
    Deserialize(#[from] toml::de::Error),
}
