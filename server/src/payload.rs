use blake3::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub file_size: usize,
    pub file_hash: Hash,
    pub file_name: String,
    pub service_name: String,
    pub backup_name: String,
}
