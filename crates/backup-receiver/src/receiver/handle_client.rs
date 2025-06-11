use core::{net::SocketAddr, time::Duration};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, ErrorKind, Write},
    time::Instant,
};

use chrono::Utc;
use shared::{Metadata, Response};
use tracing::{error, info, warn};

use crate::Context;

use super::Receiver;

impl Receiver {
    /// Handle a client connection
    pub fn handle_client<Read: BufRead>(
        &mut self,
        context: &mut Context,
        stream: &mut Read,
        peer: SocketAddr,
    ) -> Result<Metadata, Response> {
        context.current_context = "Handle Client";

        // Apply rate limit
        if let Some(history) = self.history.get_mut(&peer.ip()) {
            history.retain(|backup_time| backup_time.elapsed() < Duration::from_secs(60 * 60));

            if history.len() >= self.config.limits.maximum_backups_per_hour {
                warn!("{context}Exceeded rate limit");
                return Err(Response::ExceededRateLimit);
            }
        }

        // Read metadata
        let metadata = {
            context.current_context = "Read Metadata";

            let mut buffer = [0u8; size_of::<Metadata>()];

            // Read bytes
            stream
                .read_exact(&mut buffer)
                .map_err(|error| match error.kind() {
                    ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                        warn!("{context}Timed out");
                        Response::Timeout
                    }
                    ErrorKind::UnexpectedEof => {
                        warn!("{context}Unexpected Eof");
                        Response::BadData
                    }
                    _ => {
                        error!("{context}Encountered error: {error}");
                        Response::Error
                    }
                })?;

            // Try cast the bytes to a Metadata instance.
            let metadata = Metadata::try_from(buffer.as_slice())
                .inspect_err(|e| warn!("{context}Invalid metadata: {e}"))
                .map_err(|_| Response::BadData)?;

            context.backup = Some((metadata.service_name.as_string(), metadata.cadence));
            info!("{context}Received metadata");

            metadata
        };

        // Check limits
        let backup_bytes = {
            if metadata.backup_bytes > self.config.limits.maximum_payload_bytes {
                warn!(
                    "{context}Exceeded payload size limit {} > {}",
                    metadata.backup_bytes, self.config.limits.maximum_payload_bytes
                );
                return Err(Response::TooLarge);
            }

            usize::try_from(metadata.backup_bytes)
                .inspect_err(|e| {
                    error!(
                        "{context}Backup bytes {} > usize::MAX: {e}",
                        metadata.backup_bytes
                    )
                })
                .map_err(|_| Response::Error)?
        };

        // Prepare backup file
        let mut file = {
            context.current_context = "Prepare Backup";

            let backup_directory = metadata.backup_directory();

            // Check if the backup dir exists
            let directroy_metadata = match fs::metadata(&backup_directory) {
                Ok(dir_metadata) => Some(dir_metadata),
                Err(error) => {
                    if error.kind() == ErrorKind::NotFound {
                        None
                    } else {
                        error!(
                            "{context}Could not check metadata for {backup_directory:?}: {error}"
                        );
                        return Err(Response::Error);
                    }
                }
            };

            match directroy_metadata {
                // If the backup_dir exists, ensure it is a directory
                Some(directory_metadata) => {
                    if !directory_metadata.is_dir() {
                        error!(
                            "{context}{backup_directory:?} is not a dir: {directory_metadata:?}"
                        );
                        return Err(Response::Error);
                    }
                }

                // If it does not exist, create it.
                None => {
                    fs::create_dir_all(&backup_directory)
                        .inspect_err(|e| {
                            error!("{context}Could not create directory {backup_directory:?}: {e}")
                        })
                        .map_err(|_| Response::Error)?;
                }
            }

            let file_name = format!(
                "{}.{}",
                Utc::now().format("%Y-%m-%d_%H-%M-%S"),
                metadata.file_extension
            );
            let backup_file_path = backup_directory.join(file_name);

            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&backup_file_path)
                .inspect_err(|e| {
                    error!("{context}Could not create and open file at {backup_file_path:?}: {e}")
                })
                .map_err(|_| Response::Error)?
        };

        // Stream payload into file
        {
            context.current_context = "Read Write Payload";

            let start = Instant::now();

            // Setup 1 KiB buffer for reading
            let mut file_buffer = [0u8; 1024];
            let mut total_bytes_read: usize = 0;

            // Read the payload in chunks and append the chunks to the output file.
            while total_bytes_read < backup_bytes {
                if start.elapsed().as_secs() > self.config.limits.timeout_seconds {
                    warn!("{context}Timed out receiving payload.");
                    return Err(Response::Timeout);
                }

                let bytes_read = match stream.read(&mut file_buffer[..]) {
                    Ok(bytes) => bytes,
                    Err(e) => match e.kind() {
                        ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                            continue;
                        }
                        _ => {
                            error!("{context}Encountered error: {e}");
                            return Err(Response::Error);
                        }
                    },
                };

                file.write_all(&file_buffer[..bytes_read])
                    .inspect_err(|e| {
                        error!("{context}Encountered error when writing to backup file: {e}")
                    })
                    .map_err(|_| Response::Error)?;

                total_bytes_read += bytes_read;
            }

            info!("{context}Saved backup");
        }

        Ok(metadata)
    }
}
