use std::{fs, io::Read, net::TcpStream, path::PathBuf};

use anyhow::{bail, Context};
use rustls::{ServerConnection, Stream};
use shared::TlsPayload;

use crate::{cleanup::cleanup, BACKUP_PATH};

pub fn handler(stream: &mut Stream<'_, ServerConnection, TcpStream>) -> anyhow::Result<()> {
    // payload size hint
    let mut payload_size: [u8; 8] = [0; 8];
    stream
        .read_exact(&mut payload_size)
        .context("Failed to read paylod size")?;
    let payload_size: usize = usize::from_be_bytes(payload_size);

    // payload
    let mut payload_buffer: Vec<u8> = vec![0; payload_size];
    stream
        .read_exact(&mut payload_buffer)
        .context("Failed to read payload")?;
    let payload_string = String::from_utf8_lossy(&payload_buffer);
    let payload: TlsPayload =
        toml::from_str(&payload_string).context("Failed to deserialize payload")?;

    // file
    let mut file: Vec<u8> = vec![0; payload.file_size];
    stream
        .read_exact(&mut file)
        .context("Failed to read backup")?;

    // write file
    let backup_path = PathBuf::from(BACKUP_PATH)
        .join(&payload.service_name)
        .join(&payload.backup_name);
    fs::create_dir_all(&backup_path).context("Failed to create backup directory")?;

    let backup_path = backup_path.join(payload.file_name);
    fs::write(&backup_path, file).context("Failed to write backup file")?;

    // verify file
    let contents = fs::read(&backup_path).context("Failed to read backup file")?;
    let hash = blake3::hash(&contents);
    if hash != payload.file_hash {
        bail!("Hash mismatch");
    }

    // cleanup
    cleanup(
        &payload.service_name,
        &payload.backup_name,
        payload.max_files,
    )
    .context("Failed to cleanup files")?;

    Ok(())
}
