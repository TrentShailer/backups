use log::error;
use rustls::RootCertStore;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::{io, net::SocketAddr, path::PathBuf};
use thiserror::Error;

use crate::certificate::{load_age_key, load_certs, load_key, load_roots};

pub struct TlsConfig {
    pub socket_address: SocketAddr,
    pub root_ca: RootCertStore,
    pub certificate_chain: Vec<CertificateDer<'static>>,
    pub private_key: PrivateKeyDer<'static>,
}

impl From<&RawConfig> for TlsConfig {
    fn from(value: &RawConfig) -> Self {
        let certificate_chain = match load_certs(&value.certificate_path) {
            Ok(v) => v,
            Err(error) => {
                error!("{}", error);
                panic!("Failed to load certificate chain");
            }
        };
        let private_key = match load_key(&value.private_key_path) {
            Ok(v) => v,
            Err(error) => {
                error!("{}", error);
                panic!("Failed to load private key");
            }
        };
        let root_ca = match load_roots(&value.root_ca_path) {
            Ok(v) => v,
            Err(error) => {
                error!("{}", error);
                panic!("Failed to load root ca");
            }
        };

        Self {
            certificate_chain,
            private_key,
            root_ca,
            socket_address: value.socket_address,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub backup_path: PathBuf,
    pub age_key: age::x25519::Identity,
    pub service_config: Vec<ServiceConfig>,
}

impl From<&RawConfig> for Config {
    fn from(value: &RawConfig) -> Self {
        let age_key = match load_age_key(&value.age_key_path) {
            Ok(v) => v,
            Err(error) => {
                error!("{}", error);
                panic!("Failed to load age key");
            }
        };

        Self {
            backup_path: value.backup_path.clone(),
            age_key,
            service_config: value.service_config.clone(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct RawConfig {
    pub socket_address: SocketAddr,
    pub backup_path: PathBuf,
    pub root_ca_path: PathBuf,
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
    pub age_key_path: PathBuf,
    pub service_config: Vec<ServiceConfig>,
}

#[derive(Deserialize, Clone)]
pub struct ServiceConfig {
    pub folder_name: String,
    pub backup_configs: Vec<BackupConfig>,
}

#[derive(Deserialize, Clone)]
pub struct BackupConfig {
    pub folder_name: String,
    pub max_files: i32,
}

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] io::Error),
    #[error("Failed to deserialze config file: {0}")]
    TomlDeserialzeError(#[from] toml::de::Error),
}
