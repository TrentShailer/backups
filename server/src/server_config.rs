use std::{fs, net::SocketAddr, path::PathBuf};

use error_trace::{ErrorTrace, ResultExt};
use serde::{Deserialize, Serialize};

use crate::BLOCKLIST_PATH;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub socket_address: SocketAddr,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub key_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blocklist {
    pub blocked_ips: Vec<SocketAddr>,
}

impl Blocklist {
    pub fn new() -> Self {
        Self {
            blocked_ips: vec![],
        }
    }

    pub fn save(&self) -> Result<(), ErrorTrace> {
        let contents = toml::to_string_pretty(self).track()?;
        fs::write(BLOCKLIST_PATH, contents).track()?;
        Ok(())
    }
}
