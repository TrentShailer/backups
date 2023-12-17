use std::{io, str::Utf8Error};

use blake3::Hash;
use serde::Deserialize;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

#[derive(Deserialize)]
pub struct PayloadConfig {
    pub folder: String,
    pub sub_folder: String,
    pub file_name: String,
    pub file_hash: Hash,
    pub file_size: usize,
}

pub async fn get_payload_config(
    reader: &mut ReadHalf<&mut TlsStream<TcpStream>>,
) -> Result<PayloadConfig, GetPayloadConfigError> {
    let mut contents: Vec<u8> = vec![0; 1024];

    let buffer_size = reader
        .read(&mut contents)
        .await
        .map_err(GetPayloadConfigError::ReadStreamError)?;

    let contents_string = std::str::from_utf8(&contents[0..buffer_size])?;

    let payload_config: PayloadConfig = toml::de::from_str(contents_string)?;

    Ok(payload_config)
}

#[derive(Debug, Error)]
pub enum GetPayloadConfigError {
    #[error("ReadStreamError[br]{0}")]
    ReadStreamError(#[source] io::Error),
    #[error("ConvertToStringError[br]{0}")]
    ConvertToStringError(#[from] Utf8Error),
    #[error("DeserialzeError[br]{0}")]
    DeserializeError(#[from] toml::de::Error),
}
