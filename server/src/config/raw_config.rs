use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;

use super::ServiceConfig;

#[derive(Deserialize, Clone)]
pub struct RawConfig {
    pub socket_address: SocketAddr,
    pub backup_path: PathBuf,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
    pub service_config: Vec<ServiceConfig>,
}
