use std::{fs, path::PathBuf, time::SystemTime};

use error_trace::{ErrorTrace, ResultExt};

use crate::BACKUP_PATH;

pub fn cleanup(service_name: &str, backup_name: &str, max_files: usize) -> Result<(), ErrorTrace> {
    let path = PathBuf::from(BACKUP_PATH)
        .join(service_name)
        .join(backup_name);

    let mut entries = fs::read_dir(path).track()?;
    let mut files: Vec<(SystemTime, PathBuf)> = Vec::new();

    while let Some(entry) = entries.next() {
        let entry = entry.track()?;
        let metadata = entry.metadata().track()?;

        if !metadata.is_file() {
            continue;
        }

        let created = metadata.created().track()?;

        files.push((created, entry.path()));
    }

    if files.len() < max_files {
        return Ok(());
    }

    files.sort_by(|a, b| b.0.cmp(&a.0));

    let files_to_delete = files.len() - max_files;

    for _ in 0..files_to_delete {
        let file = files.pop().unwrap();

        fs::remove_file(file.1).track()?;
    }

    Ok(())
}
