use std::collections::HashMap;

use crate::{State, theme, unicode_icon};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Tag {
    name: String,
    color: u32,
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

pub fn render_tag_list(state: &mut State, outer_block: &Block, area: &Rect, frame: &mut Frame) {
    let list = List::new(state.data.tags.map().values().enumerate().map(|(i, l)| {
        let name = l.name();
        let icon = unicode_icon(0xf1224, Color::from_u32(*l.color()));
        let ln = Line::from(vec![icon, Span::raw(name)]);

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
