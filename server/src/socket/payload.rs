use std::io;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use super::payload_config::PayloadConfig;

pub async fn get_payload(
    reader: &mut ReadHalf<&mut TlsStream<TcpStream>>,
    payload_config: &PayloadConfig,
) -> Result<Vec<u8>, GetPayloadError> {
    let mut contents: Vec<u8> = vec![0; payload_config.file_size + 1024];

    reader
        .read(&mut contents)
        .await
        .map_err(GetPayloadError::ReadStreamError)?;

    Ok(contents)
}

#[derive(Debug, Error)]
pub enum GetPayloadError {
    #[error("ReadStreamError[br]{0}")]
    ReadStreamError(#[source] io::Error),
}
