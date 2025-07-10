use crate::{State, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

use tachyonfx::ToRgbComponents;
#[allow(unused_imports)]
use tracing::{info, warn};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Tag {
    name: String,
    color: u32,
    pub refs: i32,
}

impl Tag {
    pub fn color(&self) -> &u32 {
        &self.color
    }
    pub fn name(&self) -> &str {
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

    pub fn rm_ref(&mut self, tag_str: &str) -> bool {
        let tag_idx = self
            .tags
            .iter_mut()
            .position(|t| t.name == tag_str)
            .unwrap();

        let tag = self.tags.get_mut(tag_idx).unwrap();
        tag.refs -= 1;
        if tag.refs == 0 {
            self.tags.remove(tag_idx);
            return true;
        }
        false
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}

fn rgb_to_hex(color: (u8, u8, u8)) -> u32 {
    let (r, g, b) = color;
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn handle_edit(state: &mut State, input: String) {
    let color_regex: &str = &theme::TERM_COLORS_REGEX;
    let full = format!(r"^\w+:\s((#\w{{6}})|{color_regex})");
    let check = Regex::new(&full).unwrap();
    if !check.is_match(&input) {
        warn!("Wrong format for tag edit");
        info!("regex: {}", full);
        state.popup_msg = Span::styled("Bad Input", theme::RED);
        state.popup_active = true;
        return;
    }

    let tag = state
        .data
        .tags
        .tags
        .get_mut(state.list_state.selected().unwrap())
        .unwrap();

    let (new_name, color_str_org) = input.split_once(":").unwrap();
    let color_str = color_str_org.replace(" #", "");
    let color = if color_str_org.contains("#") {
        u32::from_str_radix(&color_str, 16).unwrap()
    } else {
        info!("color str: {}", color_str);
        rgb_to_hex(theme::TERM_COLORS.get(color_str.trim()).unwrap().to_rgb())
    };

    let iter = state
        .data
        .logs
        .iter_mut()
        .filter(|l| l.tags.contains(tag.name()));
    for log in iter {
        log.tags.remove(tag.name());
        log.tags.insert(new_name.to_string());
    }

    tag.name = new_name.to_string();
    tag.color = color;
}

pub fn render_tag_list(
    state: &mut State,
    outer_block: &Block,
    area: &Rect,
    frame: &mut Frame,
) {
    let list = List::new(state.data.tags.tags().iter().map(|l| {
        let name = l.name();
        let icon = theme::unicode_icon(0xf1224, Color::from_u32(*l.color()));
        let ln = Line::from(vec![
            icon,
            Span::raw(name),
            Span::styled(
                format!("{}{}", " ".repeat(20 - name.len()), l.refs),
                theme::BLUE,
            ),
        ]);

        let color = theme::BG0;
        ListItem::from(ln).bg(color)
    }))
    .block(outer_block.clone())
    .fg(theme::TEXT)
    .bg(theme::BG0)
    .highlight_style(Style::default().bg(theme::BG1))
    .highlight_symbol("> ");

    frame.render_stateful_widget(list, *area, &mut state.list_state);
}
