use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, List, ListState},
};

use crate::{State, theme};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    #[default]
    LOG,
    TAG,
    PASTLOG,
}

impl ListType {
    pub const TYPES: [ListType; 3] = [ListType::LOG, ListType::TAG, ListType::PASTLOG];

    fn to_span(&self) -> Line {
        match self {
            ListType::LOG => {
                let icon = theme::unicode_icon(0xf02c, Color::Blue);
                let name = Span::raw("Logs");
                Line::from(vec![icon, name])
            }
            ListType::PASTLOG => {
                let icon = theme::unicode_icon(0xf02c, Color::Blue);
                let name = Span::raw("Past Logs");
                Line::from(vec![icon, name])
            }
            ListType::TAG => {
                let icon = theme::unicode_icon(0xf02c, Color::Magenta);
                let name = Span::raw("Tags");
                Line::from(vec![icon, name])
            }
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            ListType::LOG => "| Logs |",
            ListType::TAG => "| Tags |",
            ListType::PASTLOG => "| Past Logs |",
        }
    }
}

pub fn render_tab_list(area: &Rect, state: &State, frame: &mut Frame) {
    let tab_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::ORANG)
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
