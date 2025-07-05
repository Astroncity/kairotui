use crate::{ListType, State, tag::TagSys, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, List, ListItem},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Instant};

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub done: bool,
    pub name: String,
    pub text: String,
    #[serde(with = "serde_millis")]
    pub start: Instant,
    #[serde(with = "serde_millis")]
    pub end: Instant,
    pub tags: HashSet<String>,
}

impl Log {
    pub fn new(desc: String, tags: HashSet<String>) -> Self {
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
    let matches: Vec<&str> =
        regex.find_iter(&input).map(|m| m.as_str()).collect();
    let mut cpy = input.clone();

    for m in &matches {
        cpy = input.replace(m, "");
    }

    (
        cpy.trim().to_owned(),
        matches
            .iter()
            .map(|m| m.to_string().replace("tag: ", ""))
            .collect(),
    )
}

pub fn handle_add(state: &mut State, input: String) {
    let (name, tags) = parse_input(input);
    tags.iter().for_each(|t| state.data.tags.add(t).refs += 1);
    state
        .data
        .items
        .push(Log::new(name, HashSet::from_iter(tags)));
}

fn get_log_tag_text<'a>(log: &'a Log, sys: &'a TagSys) -> Vec<Span<'a>> {
    let mut spans: Vec<Span> = Vec::new();
    for t in &log.tags {
        let str = String::from(" ") + t;
        let tag = sys.tags().iter().find(|e| e.name() == t).unwrap();
        let color = Color::from_u32(*tag.color());
        spans.push(Span::styled(str, color));
    }
    spans
}

pub fn render_log_list(
    state: &mut State,
    outer_block: &Block,
    area: &Rect,
    frame: &mut Frame,
) {
    let list = List::new(state.data.items.iter().enumerate().map(|(i, l)| {
        let v = if l.done {
            l.text.to_span().crossed_out()
        } else {
            l.text.to_span()
        };
        let tag_txt = get_log_tag_text(l, &state.data.tags);
        let ln = Line::from([&vec![v][..], &tag_txt[..]].concat());
        let color = if i % 2 == 0 { theme::BG0 } else { theme::BG1 };
        ListItem::from(ln).bg(color)
    }))
    .block(outer_block.clone())
    .fg(theme::TEXT)
    .bg(theme::BG0)
    .highlight_style(Style::default().fg(theme::ORANG))
    .highlight_symbol("> ");

    frame.render_stateful_widget(list, *area, &mut state.list_state);
}
