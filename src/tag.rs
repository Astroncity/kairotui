use std::collections::HashMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Tag {
    name: String,
    color: u32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct TagSys {
    map: HashMap<String, Tag>,
}

impl TagSys {
    pub fn add(self: &mut Self, name: &str) {
        self.map.insert(
            name.to_string(),
            Tag {
                name: name.to_string(),
                color: 0x00ff0000,
            },
        );
    }
}
