use std::{collections::HashMap, hash::Hash};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Tag {
    name: String,
    color: u32,
}

impl Tag {
    pub fn color(self: &Self) -> u32 {
        self.color
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
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

    pub fn map(self: &Self) -> &HashMap<String, Tag> {
        &self.map
    }
}
