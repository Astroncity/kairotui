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
    assert!(state.curr_list() == ListType::LOG);
    if let Some(i) = state.list_state.selected() {
        state.data.items.remove(i);
    }
}

pub fn parse_input(state: &mut State) -> (String, Vec<String>) {
    assert!(state.curr_list() == ListType::LOG);
    let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();
    let matches: Vec<&str> = regex.find_iter(&state.input).map(|m| m.as_str()).collect();
    let mut log_name = state.input.clone();

    for m in &matches {
        log_name = log_name.replace(m, "");
    }

    (
        log_name.trim().to_owned(),
        matches.iter().map(|m| m.to_string().replace("tag: ", "")).collect(),
    )
}

pub fn handle_add(key: KeyEvent, state: &mut State) -> bool {
    assert!(state.curr_list() == ListType::LOG);
    match key.code {
        event::KeyCode::Enter => {
            let (name, tags) = parse_input(state);
            tags.iter().for_each(|t| state.data.tags.add(t));
            state.data.items.push(Log::new(name, tags));
            state.input.clear();
            state.update_input_display();
            return true;
        }
        event::KeyCode::Esc => return true,
        event::KeyCode::Backspace => {
            state.input.pop();
        }
        event::KeyCode::Char(c) => {
            state.input.push(c);
        }
        _ => {}
    }
    state.update_input_display();
    false
}
