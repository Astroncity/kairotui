use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{log::Log, tag::TagSys};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersistentData {
    pub opened_once: bool,
    pub logs: Vec<Log>,
    pub past_logs: Vec<Log>,
    pub tags: TagSys,
    pub save_path: Option<String>,
}

impl PersistentData {
    pub fn new(path: String) -> Self {
        Self {
            save_path: Some(path),
            ..Default::default()
        }
    }

    pub fn save(self: &Self) -> Result<()> {
        let data = serde_json::to_string(self).unwrap();
        if let Some(path) = &self.save_path {
            fs::write(path, data)?;
        } else {
            anyhow::bail!("no save path set.");
        }
        Ok(())
    }

    pub fn load(self: &Self) -> Result<PersistentData> {
        if let Some(path) = &self.save_path {
            let str: String = fs::read_to_string(path)?;
            let dat: PersistentData = serde_json::from_str(&str)?;
            Ok(dat)
        } else {
            anyhow::bail!("no save path set.");
        }
    }
}
