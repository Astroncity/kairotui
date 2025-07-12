use std::{cell::RefCell, rc::Rc};

use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, List, ListState},
};

use crate::{
    State,
    data::{self, SaveData},
    theme,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    #[default]
    Log,
    Tag,
    PastLog,
}

pub trait Tab {
    fn render(
        self: &mut Self,
        block: &Block,
        area: &Rect,
        frame: &mut Frame,
        data: &mut SaveData,
    );
    fn get_title(self: &Self) -> &str;
    fn get_line(self: &Self) -> Line<'static>;
    fn handle_keys(self: &mut Self, key: KeyCode, data: &mut SaveData);
}

impl<'a> ListType {
    pub const TYPES: [ListType; 3] = [ListType::Log, ListType::Tag, ListType::PastLog];

    fn to_span(self) -> Line<'a> {
        match self {
            ListType::Log => {
                let icon = theme::unicode_icon(0xf02c, theme::BLUE);
                let name = Span::raw("Logs");
                Line::from(vec![icon, name])
            }
            ListType::PastLog => {
                let icon = theme::unicode_icon(0xf02c, theme::AQUA);
                let name = Span::raw("Past Logs");
                Line::from(vec![icon, name])
            }
            ListType::Tag => {
                let icon = theme::unicode_icon(0xf02c, theme::RED);
                let name = Span::raw("Tags");
                Line::from(vec![icon, name])
            }
        }
    }
    pub fn to_str(self) -> &'a str {
        match self {
            ListType::Log => "| Logs |",
            ListType::Tag => "| Tags |",
            ListType::PastLog => "| Past Logs |",
        }
    }
}

pub fn render_tab_list(area: &Rect, state: &State, frame: &mut Frame) {
    let tab_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::ORANGE)
        .bg(theme::BG0)
        .title("| Tabs |".to_span().into_centered_line());

    let tab_lines = ListType::TYPES.iter().map(|t| t.to_span());

    let tab_list = List::new(tab_lines)
        .block(tab_block)
        .fg(theme::TEXT)
        .bg(theme::BG0)
        .highlight_style(Style::default().bg(theme::BG1));

    let mut st = ListState::default().with_selected(Some(state.focused_list_idx));

    frame.render_stateful_widget(tab_list, *area, &mut st);
}
