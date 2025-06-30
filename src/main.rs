mod animation;
mod log;
mod tag;
mod theme;

use crate::log::Log;

use animation::AnimationHandler;

use crate::tag::*;
use anyhow::{Ok, Result};
use dirs::config_dir;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Widget},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, Instant};
use tachyonfx::{Duration as FxDuration, fx::sweep_in};

#[derive(Debug, Serialize, Deserialize, Default)]
struct PersistentData {
    items: Vec<Log>,
    tags: TagSys,
    save_path: Option<String>,
}

impl PersistentData {
    fn new(path: String) -> Self {
        Self {
            save_path: Some(path),
            ..Default::default()
        }
    }

    fn save(self: &Self) -> Result<()> {
        let data = serde_json::to_string(self).unwrap();
        if let Some(path) = &self.save_path {
            fs::write(path, data)?;
        } else {
            anyhow::bail!("no save path set.");
        }
        Ok(())
    }

    fn load(self: &Self) -> Result<PersistentData> {
        if let Some(path) = &self.save_path {
            let str: String = fs::read_to_string(path)?;
            let dat: PersistentData = serde_json::from_str(&str)?;
            Ok(dat)
        } else {
            anyhow::bail!("no save path set.");
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ListType {
    #[default]
    LOG,
    TAG,
}

impl std::fmt::Display for ListType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ListType::LOG => write!(f, "Log"),
            ListType::TAG => write!(f, "Tag"),
        }
    }
}

impl ListType {
    pub const TYPES: [ListType; 2] = [ListType::LOG, ListType::TAG];
}

#[derive(Debug, Default)]
struct State {
    data: PersistentData,
    list_state: ListState,
    adding_log: bool,
    input: String,
    input_display: Line<'static>,
    anims: AnimationHandler,
    input_anim: usize,
    focused_list: usize,
    dt: f64,
}

impl State {
    pub fn update_input_display(&mut self) {
        let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();

        self.input_display = if regex.is_match(&self.input) {
            let index = self.input.chars().position(|c| c == 't').unwrap_or(0);

            Line::from(vec![
                Span::styled(self.input[..index].to_string(), theme::TEXT),
                Span::styled(self.input[index..].to_string(), theme::ORANG),
            ])
        } else {
            Line::from(vec![Span::raw(self.input.clone())])
        };
    }

    pub fn curr_list(self: &Self) -> ListType {
        ListType::TYPES[self.focused_list]
    }
}

fn main() -> Result<()> {
    let mut state = init()?;
    state.input_anim = state.anims.add(
        sweep_in(
            tachyonfx::Motion::LeftToRight,
            16,
            0,
            theme::BG0,
            FxDuration::from_millis(500),
        ),
        Rect::default(),
    );

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();
    result?;
    Ok(())
}

fn init() -> Result<State> {
    let mut state = State::default();
    let mut data_path = config_dir().unwrap();
    data_path.push("kairotui");
    fs::create_dir_all(&data_path)?;
    data_path.push("save.dat");
    state.data.save_path = Some(data_path.to_str().unwrap().to_string());

    let _ = color_eyre::install();

    if fs::exists(&data_path)? {
        state.data = state.data.load()?;
    } else {
        state.data = PersistentData::new(data_path.to_str().unwrap().to_owned());
    }
    Ok(state)
}

fn delegate_enter(state: &mut State) {
    match state.curr_list() {
        ListType::LOG => {
            if let Some(i) = state.list_state.selected() {
                state.data.items[i].done = !state.data.items[i].done;
            }
        }
        ListType::TAG => {}
    }
}

fn handle_key(key: KeyEvent, state: &mut State) -> bool {
    match key.code {
        event::KeyCode::Char(char) => match char {
            'q' => return true,
            'A' => {
                if state.curr_list() == ListType::LOG {
                    state.adding_log = true;
                }
            }
            'D' => {
                if state.curr_list() == ListType::LOG {
                    log::delete_selected(state);
                }
            }
            'n' | 'j' => {
                state.list_state.select_next();
            }
            'm' | 'k' => {
                state.list_state.select_previous();
            }
            _ => {}
        },
        event::KeyCode::Enter => {
            delegate_enter(state);
        }
        event::KeyCode::Tab => {
            state.focused_list += 1;
            state.focused_list %= ListType::TYPES.len();
            state.list_state.scroll_up_by(u16::MAX);
        }
        _ => {}
    }
    false
}

fn handle_event(state: &mut State) -> bool {
    let _ = state.data.save();
    if let Event::Key(key) = event::read().unwrap() {
        if state.adding_log {
            if log::handle_add(key, state) {
                state.adding_log = false;
            }
        } else {
            return handle_key(key, state);
        }
    }
    false
}

fn run(mut terminal: DefaultTerminal, state: &mut State) -> Result<()> {
    let mut last_frame = std::time::Instant::now();

    loop {
        let now = Instant::now();
        state.dt = now.duration_since(last_frame).as_secs_f64();
        last_frame = now;

        update_logs(&mut state.data.items);
        terminal.draw(|x| render(x, state))?;

        let timeout = if state.anims.running() {
            Duration::from_millis(32)
        } else {
            Duration::from_millis(500)
        };

        if event::poll(timeout)? {
            if handle_event(state) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    Ok(())
}

fn compute_main_layout(frame: &Frame, state: &State) -> (Rect, Rect, Option<Rect>) {
    let [tabs_and_main] = Layout::vertical([Constraint::Fill(1)]).margin(1).areas(frame.area());

    let [tab_area, main_area] = Layout::horizontal([
        Constraint::Length(20), // Width of tab area
        Constraint::Min(10),
    ])
    .areas(tabs_and_main);

    if state.adding_log {
        let [todo_area, input_area] = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).areas(main_area);
        (tab_area, todo_area, Some(input_area))
    } else {
        let [todo_area] = Layout::vertical([Constraint::Fill(1)]).areas(main_area);
        (tab_area, todo_area, None)
    }
}

fn render_input_window(area: Rect, state: &mut State, frame: &mut Frame) {
    Paragraph::new(state.input_display.clone())
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(theme::ORANG)
                .bg(theme::BG0)
                .title(" New Task "),
        )
        .fg(theme::TEXT)
        .render(area, frame.buffer_mut());
}

fn duration_as_hhmmss(dur: Duration) -> String {
    let total_seconds = dur.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// TODO: Optimize
fn update_logs(logs: &mut [Log]) {
    for log in logs.iter_mut().filter(|l| !l.done) {
        log.end = Instant::now();
        let dur = log.end.duration_since(log.start);
        log.text = format!("{} {}", log.name, duration_as_hhmmss(dur));
    }
}

fn get_log_tag_text<'a>(log: &'a Log, tag_sys: &'a TagSys) -> Vec<Span<'a>> {
    let mut spans: Vec<Span> = Vec::new();
    for t in &log.tags {
        let str = String::from(" ") + t;
        let color = Color::from_u32(*tag_sys.map()[t].color());
        spans.push(Span::styled(str, color));
    }
    spans
}

fn render_todo_list(state: &mut State, outer_block: &Block, area: &Rect, frame: &mut Frame) {
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

fn render_tab_list(area: &Rect, state: &State, frame: &mut Frame) {
    let tab_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::ORANG)
        .bg(theme::BG0)
        .title(" Tabs ".to_span().into_centered_line());

    let tab_str = ListType::TYPES.iter().map(|t| t.to_string());

    let tab_list = List::new(tab_str.map(|s| ListItem::from(Span::raw(s))))
        .block(tab_block)
        .fg(theme::TEXT)
        .bg(theme::BG0)
        .highlight_style(Style::default().bg(theme::BG1));

    let mut st = ListState::default().with_selected(Some(state.focused_list));

    frame.render_stateful_widget(tab_list, *area, &mut st);
}

fn render_main_screen(frame: &mut Frame, state: &mut State) -> Option<Rect> {
    let (tab_area, todo_area, input_area) = compute_main_layout(frame, state);

    let outer_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::BLUE)
        .bg(theme::BG0)
        .title(" Logs ".to_span().into_centered_line());

    match ListType::TYPES[state.focused_list] {
        ListType::LOG => render_todo_list(state, &outer_block, &todo_area, frame),
        ListType::TAG => render_tag_list(state, &outer_block, &todo_area, frame),
    }

    render_tab_list(&tab_area, state, frame);

    input_area
}

fn render(frame: &mut Frame, state: &mut State) {
    let input_area = render_main_screen(frame, state);

    if state.adding_log {
        assert!(input_area.is_some(), "input area not ready");
        let area = input_area.unwrap();
        render_input_window(area, state, frame);
        state.anims.set_progress(true, state.input_anim, area);
    } else {
        state.anims.reset_anim(state.input_anim);
        state.anims.set_progress(false, state.input_anim, Rect::default());
    }

    state.anims.progress(frame, state.dt);
}
