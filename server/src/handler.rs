use std::{fs, io::Read, net::TcpStream, path::PathBuf};

use error_trace::{ErrorTrace, ResultExt};
use rustls::{ServerConnection, Stream};
use shared::TlsPayload;

use crate::{cleanup::cleanup, BACKUP_PATH};

pub fn handler(stream: &mut Stream<'_, ServerConnection, TcpStream>) -> Result<(), ErrorTrace> {
    // payload size hint
    let mut payload_size: [u8; 8] = [0; 8];
    stream.read_exact(&mut payload_size).track()?;
    let payload_size: usize = usize::from_be_bytes(payload_size);

    // payload
    let mut payload_buffer: Vec<u8> = vec![0; payload_size];
    stream.read_exact(&mut payload_buffer).track()?;
    let payload_string = String::from_utf8_lossy(&payload_buffer);
    let payload: TlsPayload = toml::from_str(&payload_string).track()?;

    // file
    let mut file: Vec<u8> = vec![0; payload.file_size];
    stream.read_exact(&mut file).track()?;

    // write file
    let backup_path = PathBuf::from(BACKUP_PATH)
        .join(&payload.service_name)
        .join(&payload.backup_name);
    fs::create_dir_all(&backup_path).track()?;

    let backup_path = backup_path.join(payload.file_name);
    fs::write(&backup_path, file).track()?;

    // verify file
    let contents = fs::read(&backup_path).track()?;
    let hash = blake3::hash(&contents);
    if hash != payload.file_hash {
        return Err("Hash mismatch").track();
    }

    // cleanup
    cleanup(
        &payload.service_name,
        &payload.backup_name,
        payload.max_files,
    )
    .context("Cleanup")?;

    Ok(())
}
