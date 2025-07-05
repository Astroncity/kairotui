use crate::{State, theme, unicode_icon};

use color_eyre::owo_colors::style;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Tag {
    name: String,
    color: u32,
    pub refs: i32,
}

impl Tag {
    pub fn color(self: &Self) -> &u32 {
        &self.color
    }
    pub fn name(self: &Self) -> &str {
        &self.name
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct TagSys {
    tags: Vec<Tag>,
}

impl TagSys {
    pub fn add(&mut self, name: &str) -> &mut Tag {
        if let Some(i) = self.tags.iter().position(|t| t.name() == name) {
            return &mut self.tags[i];
        }

        let t = Tag {
            name: name.to_string(),
            color: 0x00ff0000,
            refs: 0,
        };
        self.tags.push(t);
        self.tags.last_mut().unwrap()
    }

    pub fn tags(self: &Self) -> &Vec<Tag> {
        &self.tags
    }
}

pub fn handle_edit(state: &mut State, input: String) {
    let check = Regex::new(r"^\w+:\s#\w{6}$").unwrap();
    if !check.is_match(&input) {
        warn!("Wrong format for tag edit");
        return;
    }

    let tag = state
        .data
        .tags
        .tags
        .get_mut(state.list_state.selected().unwrap())
        .unwrap();

    let (new_name, color_str_org) = input.split_once(":").unwrap();
    let color_str = color_str_org.replace(": #", "");
    info!("{}", color_str);
    let color = color_str.trim().parse().unwrap();

    let iter = state.data.items.iter_mut().filter(|l| l.tags.contains(tag.name()));
    for log in iter {
        log.tags.remove(tag.name());
        log.tags.insert(new_name.to_string());
    }

    tag.name = new_name.to_string();
    tag.color = color;
}

pub fn render_tag_list(state: &mut State, outer_block: &Block, area: &Rect, frame: &mut Frame) {
    let list = List::new(state.data.tags.tags().into_iter().enumerate().map(|(i, l)| {
        let name = l.name();
        let icon = unicode_icon(0xf1224, Color::from_u32(*l.color()));
        let ln = Line::from(vec![
            icon,
            Span::raw(name),
            Span::styled(format!("{}{}", " ".repeat(20 - name.len()), l.refs), theme::BLUE),
        ]);

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
