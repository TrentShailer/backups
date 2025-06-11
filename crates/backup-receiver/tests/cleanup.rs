//! Tests for cleanup
//!

use std::fs;

use backup_receiver::{Config, Context, cleanup};
use common::clear_backups;
use shared::{Cadence, Metadata, MetadataString};

mod common;

#[test]
fn cleanup_max_files() {
    let metadata = Metadata::new(
        512,
        MetadataString::try_from("cleanup_max_files").unwrap(),
        Cadence::Daily,
        MetadataString::try_from("test").unwrap(),
    );
    clear_backups(&metadata);

    let backup_directory = metadata.backup_directory();
    let config = Config::default();
    let max_files = config.limits.maximum_files.daily;

    fs::create_dir_all(&backup_directory).unwrap();
    for i in 0..max_files + 1 {
        let path = backup_directory.join(format!("file{i}"));
        fs::write(path, "Contents").unwrap();
    }

    let mut context = Context::default();
    cleanup(&mut context, &config, &metadata);

    let directory: Vec<_> = fs::read_dir(backup_directory).unwrap().collect();
    assert_eq!(directory.len(), usize::try_from(max_files).unwrap());
    let file0_exists = directory
        .iter()
        .any(|file| file.as_ref().unwrap().file_name() == "file0");
    assert!(!file0_exists);

    clear_backups(&metadata);
}
