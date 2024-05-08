mod history_items;

use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
    time::SystemTime,
};

use error_trace::{ErrorTrace, ResultExt};
use serde::{Deserialize, Serialize};

use crate::scheduler_config::BackupName;

use self::history_items::EndpointHistory;

const HISTORY_PATH: &str = "./history.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub endpoints: Vec<EndpointHistory>,
}

impl History {
    pub fn init() -> Result<Self, ErrorTrace> {
        let path = PathBuf::from(HISTORY_PATH);
        if path.exists() {
            let mut file = File::open(path).context("Open file")?;
            let mut contents = String::new();
            File::read_to_string(&mut file, &mut contents).context("Read file")?;

            let history: Self = toml::from_str(&contents).context("Parse file")?;

            Ok(history)
        } else {
            Ok(Self { endpoints: vec![] })
        }
    }

    pub fn last_backed_up(&self, name: &BackupName) -> SystemTime {
        for endpoint in self.endpoints.iter() {
            if let Some(v) = endpoint.find(name) {
                return v;
            }
        }
        return SystemTime::UNIX_EPOCH;
    }

    pub fn update(&mut self, name: &BackupName) -> Result<(), ErrorTrace> {
        let mut found = false;
        for endpoint in self.endpoints.iter_mut() {
            if endpoint.endpoint_name == name.endpoint_name {
                endpoint.update(name);
                found = true;
                break;
            }
        }

        if found == false {
            let endpoint = EndpointHistory::create(name);
            self.endpoints.push(endpoint);
        }

        Ok(self.save().context("Save history")?)
    }

    fn save(&self) -> Result<(), ErrorTrace> {
        let contents = toml::to_string(self).track()?;
        fs::write(PathBuf::from(HISTORY_PATH), contents).track()?;
        Ok(())
    }
}
