use crate::{ListType, State};
use ratatui::crossterm::event::{self, KeyEvent};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub done: bool,
    pub name: String,
    pub text: String,
    #[serde(with = "serde_millis")]
    pub start: Instant,
    #[serde(with = "serde_millis")]
    pub end: Instant,
    pub tags: Vec<String>,
}

impl Log {
    pub fn new(desc: String, tags: Vec<String>) -> Self {
        Self {
            text: desc.clone(),
            start: Instant::now(),
            end: Instant::now(),
            done: false,
            name: desc,
            tags: tags,
        }
    }
}

pub fn delete_selected(state: &mut State) {
    assert!(state.focused_list == ListType::LOG);
    if let Some(i) = state.list_state.selected() {
        state.data.items.remove(i);
    }
}

fn parse_input(input: String) -> (String, Vec<String>) {
    let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();
    let matches: Vec<&str> = regex.find_iter(&input).map(|m| m.as_str()).collect();
    let mut cpy = input.clone();

    for m in &matches {
        cpy = input.replace(m, "");
    }

    (
        cpy.trim().to_owned(),
        matches.iter().map(|m| m.to_string().replace("tag: ", "")).collect(),
    )
}

pub fn handle_add(state: &mut State, input: String) {
    let (name, tags) = parse_input(input);
    tags.iter().for_each(|t| state.data.tags.add(t).refs += 1);
    state.data.items.push(Log::new(name, tags));
}
