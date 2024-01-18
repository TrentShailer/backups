use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub socket_address: SocketAddr,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub key_path: PathBuf,
}
