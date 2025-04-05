use std::{fs, io::ErrorKind, path::PathBuf, time::SystemTime};

use shared::{Cadance, Metadata};
use tracing::{error, warn};

use crate::{Config, ContextLogger};

/// Cleanup any files over the limit for this backup's directory.
pub fn cleanup(context: &mut ContextLogger, config: &Config, metadata: &Metadata) {
    context.current_context = "Cleanup";

    let max_files = match metadata.cadance {
        Cadance::Hourly => config.limits.maximum_files.hourly,
        Cadance::Daily => config.limits.maximum_files.daily,
        Cadance::Weekly => config.limits.maximum_files.weekly,
        Cadance::Monthly => config.limits.maximum_files.monthly,
    };
    let max_files = usize::try_from(max_files).unwrap_or(usize::MAX);

    let backup_directory = metadata.backup_directory();

    let directory = match fs::read_dir(&backup_directory) {
        Ok(directory) => directory,
        Err(error) => {
            if error.kind() == ErrorKind::NotFound {
                warn!("{context}Backup directory not found: {backup_directory:?}");
                return;
            } else {
                error!(
                    "{context}Could not get backup directory '{backup_directory:?}' metadata: {error}"
                );
                return;
            }
        }
    };

    // Get the created date for each file in the backup directory that can be accessed.
    let mut files: Vec<(SystemTime, PathBuf)> = directory
        .filter_map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!("{context}Could not read entry: {error}");
                    return None;
                }
            };
            let path = entry.path();

            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) => {
                    warn!("{context}Could not get entry '{path:?}' metadata: {error}",);
                    return None;
                }
            };

            if !metadata.is_file() {
                return None;
            }

            let created = metadata
                .created()
                .expect("OS should support file create date.");

            Some((created, path))
        })
        .collect();

    // Sort by age, oldest first.
    files.sort_by(|a, b| a.0.cmp(&b.0));

    // If there is less than the limit, return Ok
    if files.len() <= max_files {
        return;
    }

    // Remove files
    files[..files.len() - max_files]
        .iter()
        .for_each(|(_, file)| {
            if let Err(e) = fs::remove_file(file) {
                error!("{context}Could not remove file {file:?}: {e}");
            }
        });
}
