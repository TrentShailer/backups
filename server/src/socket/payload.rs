use std::{io, str::Utf8Error};

use blake3::Hash;
use serde::Deserialize;
use std::str;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

#[derive(Deserialize)]
pub struct Payload {
    pub folder: String,
    pub sub_folder: String,
    pub file_name: String,
    pub file_hash: Hash,
    pub file: Vec<u8>,
}

const BUFFER_SIZE: usize = 10000000; // 10Mb

pub async fn get_payload(
    reader: &mut ReadHalf<&mut TlsStream<TcpStream>>,
) -> Result<Payload, GetPayloadError> {
    let mut contents: Vec<u8> = vec![0; BUFFER_SIZE];

    let buffer_size = reader
        .read(&mut contents)
        .await
        .map_err(GetPayloadError::ReadStreamError)?;

    if buffer_size == BUFFER_SIZE {
        return Err(GetPayloadError::PayloadSizeError);
    }

    let contents_string = str::from_utf8(&contents[0..buffer_size])?;

    let payload: Payload = toml::de::from_str(contents_string)?;

    Ok(payload)
}

#[derive(Debug, Error)]
pub enum GetPayloadError {
    #[error("ReadStreamError[br]{0}")]
    ReadStreamError(#[source] io::Error),
    #[error("ConvertToStringError[br]{0}")]
    ConvertToStringError(#[from] Utf8Error),
    #[error("DeserialzeError[br]{0}")]
    DeserializeError(#[from] toml::de::Error),
    #[error("PayloadSizeError")]
    PayloadSizeError,
}
