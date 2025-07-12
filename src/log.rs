use crate::{data::SaveData, tab::Tab, tag::TagSys, theme};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
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

pub struct LogList {
    title: String,
    pub state: ListState,
}

impl LogList {
    pub fn new(title: &str) -> Self {
        let t = format!("| {} |", title);
        Self {
            title: t,
            state: ListState::default(),
        }
    }

    pub fn delete_selected(self: &Self, data: &mut SaveData) {
        if let Some(i) = self.state.selected() {
            let log = &data.logs[i];
            log.tags.iter().for_each(|t| {
                data.tags.rm_ref(t);
            });

            data.logs.remove(i);
            info!("{}", "Deleted active log");
        }
    }

    pub fn handle_add(input: String, data: &mut SaveData) {
        let (name, tags) = parse_input(input);
        tags.iter().for_each(|t| data.tags.add(t).refs += 1);
        data.logs.push(Log::new(name, HashSet::from_iter(tags)));
    }
}

impl Tab for LogList {
    fn render(
        self: &mut Self,
        blk: &Block,
        area: &Rect,
        frame: &mut Frame,
        data: &mut SaveData,
    ) {
        {
            if data.logs.is_empty() {
                render_empty_msg(frame, blk, area, false);
                return;
            }
        }

        let list = {
            List::new(data.logs.iter().map(|l| {
                let v = l.name.to_span().fg(theme::TEXT);
                let mut dur_str = String::from(" ");
                dur_str.push_str(&duration_as_hhmmss(l.end.duration_since(l.start)));

                let dur = Span::styled(dur_str, Style::default().fg(theme::BLUE));

                let mut vec = vec![v, dur];
                let mut tag_txt = get_log_tag_text(l, &data.tags);
                vec.append(&mut tag_txt);

                let ln = Line::from(vec);
                let color = theme::BG0;
                ListItem::from(ln).bg(color)
            }))
            .block(blk.clone())
            .bg(theme::BG0)
            .highlight_style(Style::default().bg(theme::BG1))
            .highlight_symbol("> ")
        };

        frame.render_stateful_widget(list, *area, &mut self.state);
    }

    fn get_title<'a>(self: &'a Self) -> &'a str {
        &self.title
    }

    fn get_line(self: &Self) -> Line<'static> {
        let icon = theme::unicode_icon(0xf02c, theme::BLUE);
        let name = Span::raw("Logs");
        Line::from(vec![icon, name])
    }

    fn handle_keys(self: &mut Self, key: KeyCode, data: &mut SaveData) {
        match key {
            event::KeyCode::Char(char) => match char {
                'D' => {
                    self.delete_selected(data);
                }
                'n' | 'j' => {
                    self.state.select_next();
                }
                'm' | 'k' => {
                    self.state.select_previous();
                }

                _ => {}
            },
            _ => {}
        }
    }
}

impl Log {
    pub fn new(desc: String, tags: HashSet<String>) -> Self {
        Self {
            start: Instant::now(),
            end: Instant::now(),
            done: false,
            name: desc,
            tags,
        }
    }
}

/*pub fn delete_past_log(self: &Self, data: &mut SaveData) {
    if let Some(i) = self.list_state.selected() {
        data.past_logs.remove(i);
        info!("{}", "Deleted old log");
    }
}*/

fn parse_input(input: String) -> (String, Vec<String>) {
    let regex = Regex::new(r"(tag:\s(\w+))+").unwrap();
    let matches: Vec<&str> = regex.find_iter(&input).map(|m| m.as_str()).collect();
    let mut cpy = input.clone();

    for m in &matches {
        cpy = cpy.replace(m, "");
    }

    (
        cpy.trim().to_owned(),
        matches
            .iter()
            .map(|m| m.to_string().replace("tag: ", ""))
            .collect(),
    )
}

fn get_log_tag_text<'a>(log: &'a Log, sys: &'a TagSys) -> Vec<Span<'a>> {
    let mut spans: Vec<Span> = Vec::new();
    for t in &log.tags {
        let str = String::from(" ") + t;
        let tag = sys.tags().iter().find(|e| e.name() == t).unwrap();
        let color = Color::from_u32(tag.color().clone());
        spans.push(Span::styled(str, Style::default().fg(color).bold()));
    }
    spans
}

#[derive(PartialEq, PartialOrd, Eq)]
pub enum LogType {
    Active,
    Past,
}

fn render_empty_msg(frame: &mut Frame, block: &Block, outer: &Rect, old: bool) {
    let msg = if old {
        "No completed logs."
    } else {
        "No active logs.\n Start by creating a log with <S-A>."
    };

    let area = {
        let vert = Layout::vertical([Constraint::Percentage(8)]).flex(Flex::Center);
        let horz = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);
        let [area] = vert.areas(*outer);
        let [area] = horz.areas(area);
        area
    };

    block.render(*outer, frame.buffer_mut());

    Paragraph::new(msg)
        .fg(theme::TEXT_ALT)
        .bg(theme::BG0)
        .centered()
        .render(area, frame.buffer_mut());
}

fn duration_as_hhmmss(dur: Duration) -> String {
    let total_seconds = dur.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

// TODO: Optimize
pub fn update_logs(logs: &mut [Log]) {
    for log in logs.iter_mut() {
        log.end = Instant::now();
    }
}
