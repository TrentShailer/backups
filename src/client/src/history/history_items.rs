use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub endpoint_name: String,
    pub service_name: String,
    pub backup_name: String,
    pub last_backed_up: SystemTime,
}

// impl EndpointHistory {
//     pub fn create(name: &BackupName) -> Self {
//         Self {
//             endpoint_name: name.endpoint_name.to_string(),
//             services: vec![ServiceHistory::create(name)],
//         }
//     }

//     pub fn find(&self, name: &BackupName) -> Option<SystemTime> {
//         for service in self.services.iter() {
//             if service.service_name == name.service_name {
//                 return service.find(name);
//             }
//         }

//         None
//     }

//     pub fn update(&mut self, name: &BackupName) {
//         for service in self.services.iter_mut() {
//             if service.service_name == name.service_name {
//                 service.update(name);
//                 return;
//             }
//         }

//         self.services.push(ServiceHistory::create(name))
//     }
// }
// impl ServiceHistory {
//     pub fn create(name: &BackupName) -> Self {
//         Self {
//             service_name: name.service_name.to_string(),
//             backups: vec![BackupHistory::create(name)],
//         }
//     }

//     pub fn find(&self, name: &BackupName) -> Option<SystemTime> {
//         for backup in self.backups.iter() {
//             if backup.backup_name == name.backup_name {
//                 return Some(backup.last_backed_up);
//             }
//         }

//         None
//     }

//     pub fn update(&mut self, name: &BackupName) {
//         for backup in self.backups.iter_mut() {
//             if backup.backup_name == name.backup_name {
//                 backup.last_backed_up = SystemTime::now();
//                 return;
//             }
//         }

//         self.backups.push(BackupHistory::create(name))
//     }
// }
// impl BackupHistory {
//     pub fn create(name: &BackupName) -> Self {
//         Self {
//             backup_name: name.backup_name.to_string(),
//             last_backed_up: SystemTime::now(),
//         }
//     }
// }
