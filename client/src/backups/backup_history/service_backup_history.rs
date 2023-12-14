use serde::{Deserialize, Serialize};

use super::backup_history::BackupHistory;

#[derive(Deserialize, Serialize, Clone)]
pub struct ServiceBackupHistory {
    pub folder_name: String,
    pub backups: Vec<BackupHistory>,
}

impl ServiceBackupHistory {
    pub fn new(folder_name: String, backup_names: Vec<String>) -> Self {
        let backups: Vec<BackupHistory> = backup_names
            .iter()
            .map(|name| BackupHistory::new(String::from(name)))
            .collect();

        Self {
            folder_name,
            backups,
        }
    }

    pub fn add_missing(&mut self, backup_names: Vec<String>) {
        for backup_name in backup_names.iter() {
            if !self
                .backups
                .iter()
                .any(|backup| &backup.folder_name == backup_name)
            {
                self.backups
                    .push(BackupHistory::new(String::from(backup_name)));
            }
        }
    }
}
