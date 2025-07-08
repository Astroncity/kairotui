use crate::{State, tab::ListType, tag::TagSys, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, List, ListItem},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    pub done: bool,
    pub name: String,
    #[serde(with = "serde_millis")]
    pub start: Instant,
    #[serde(with = "serde_millis")]
    pub end: Instant,
    pub tags: HashSet<String>,
}

impl Log {
    pub fn new(desc: String, tags: HashSet<String>) -> Self {
        Self {
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
        state.data.logs.remove(i);
        info!("{}", "Deleted active log");
    }
}

pub fn delete_past_log(state: &mut State) {
    assert!(state.focused_list == ListType::PASTLOG);
    if let Some(i) = state.list_state.selected() {
        state.data.past_logs.remove(i);
        info!("{}", "Deleted old log");
    }
}

fn parse_input(input: String) -> (String, Vec<String>) {
    let regex = Regex::new(r"(tag:\s(\w+))+").unwrap();
    let matches: Vec<&str> = regex.find_iter(&input).map(|m| m.as_str()).collect();
    let mut cpy = input.clone();
    info!("tag matches: {}", matches.len());

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
        .logs
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

#[derive(PartialEq, PartialOrd, Eq)]
pub enum LogType {
    ACTIVE,
    PAST,
}

pub fn render_log_list(
    state: &mut State,
    outer_block: &Block,
    area: &Rect,
    frame: &mut Frame,
    t: LogType,
) {
    let vec = if t == LogType::ACTIVE {
        &state.data.logs
    } else {
        &state.data.past_logs
    };

    let list = List::new(vec.iter().enumerate().map(|(i, l)| {
        let v = l.name.to_span().fg(theme::TEXT);
        let mut dur_str = String::from(" ");
        dur_str.push_str(&duration_as_hhmmss(l.end.duration_since(l.start)));

        let dur = Span::styled(dur_str, Style::default().fg(Color::Cyan));

        let mut vec = vec![v, dur];
        let mut tag_txt = get_log_tag_text(l, &state.data.tags);
        vec.append(&mut tag_txt);

        let ln = Line::from(vec);
        let color = if i % 2 == 0 { theme::BG0 } else { theme::BG1 };
        ListItem::from(ln).bg(color)
    }))
    .block(outer_block.clone())
    .bg(theme::BG0)
    .highlight_style(Style::default().fg(theme::ORANG))
    .highlight_symbol("> ");

    frame.render_stateful_widget(list, *area, &mut state.list_state);
}

fn duration_as_hhmmss(dur: Duration) -> String {
    let total_seconds = dur.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// TODO: Optimize
pub fn update_logs(logs: &mut [Log]) {
    for log in logs.iter_mut().filter(|l| !l.done) {
        log.end = Instant::now();
    }
}
