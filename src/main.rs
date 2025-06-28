mod animation;
mod tag;
mod theme;

use animation::AnimationHandler;

use crate::tag::TagSys;
use color_eyre::eyre::{Ok, Result};
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

#[derive(Default, Debug, Serialize, Deserialize)]
struct PersistentData {
    items: Vec<Log>,
    tags: TagSys,
}

impl PersistentData {
    pub const SAVE_PATH: &'static str = "/tmp/save.dat";

    fn save(self: &Self, filename: &str) -> Result<()> {
        let data = serde_json::to_string(self).unwrap();
        fs::write(filename, data)?;
        Ok(())
    }

    fn load(filename: &str) -> Result<PersistentData> {
        let str: String = fs::read_to_string(filename)?;
        let dat: PersistentData = serde_json::from_str(&str)?;
        Ok(dat)
    }
}

#[derive(Debug, Default)]
struct State {
    data: PersistentData,
    list: ListState,
    adding: bool,
    input: String,
    input_display: Line<'static>,
    anims: AnimationHandler,
    input_anim: usize,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct Log {
    done: bool,
    name: String,
    text: String,
    #[serde(with = "serde_millis")]
    start: Instant,
    #[serde(with = "serde_millis")]
    end: Instant,
    tags: Vec<String>,
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

fn main() -> Result<()> {
    let mut state = State::default();
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

    color_eyre::install()?;

    if fs::exists(PersistentData::SAVE_PATH)? {
        state.data = PersistentData::load(PersistentData::SAVE_PATH).unwrap();
    }

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();
    result
}

fn delete_entry(app: &mut State) {
    if let Some(i) = app.list.selected() {
        app.data.items.remove(i);
    }
}

fn parse_input(state: &mut State) -> (String, Vec<String>) {
    let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();
    let matches: Vec<&str> = regex.find_iter(&state.input).map(|m| m.as_str()).collect();
    let mut log_name = state.input.clone();

    for m in &matches {
        log_name = log_name.replace(m, "");
    }

    (
        log_name.trim().to_owned(),
        matches
            .iter()
            .map(|m| m.to_string().replace("tag: ", ""))
            .collect(),
    )
}

fn handle_add(key: KeyEvent, app: &mut State) -> bool {
    match key.code {
        event::KeyCode::Enter => {
            let (name, tags) = parse_input(app);
            tags.iter().for_each(|t| app.data.tags.add(t));
            app.data.items.push(Log::new(name, tags));
            app.input.clear();
            app.update_input_display();
            return true;
        }
        event::KeyCode::Esc => return true,
        event::KeyCode::Backspace => {
            app.input.pop();
        }
        event::KeyCode::Char(c) => {
            app.input.push(c);
        }
        _ => {}
    }
    app.update_input_display();
    false
}

fn handle_key(key: KeyEvent, app: &mut State) -> bool {
    match key.code {
        event::KeyCode::Char(char) => match char {
            'q' => return true,
            'A' => {
                app.adding = true;
            }
            'D' => {
                delete_entry(app);
            }
            'n' => {
                app.list.select_next();
            }
            'm' => {
                app.list.select_previous();
            }
            _ => {}
        },
        event::KeyCode::Enter => {
            if let Some(i) = app.list.selected() {
                app.data.items[i].done = !app.data.items[i].done;
            }
        }
        _ => {}
    }
    false
}

fn handle_event(state: &mut State) -> bool {
    let _ = state.data.save(PersistentData::SAVE_PATH);
    if let Event::Key(key) = event::read().unwrap() {
        if state.adding {
            if handle_add(key, state) {
                state.adding = false;
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
    let [tabs_and_main] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [tab_area, main_area] = Layout::horizontal([
        Constraint::Length(20), // Width of tab area
        Constraint::Min(10),
    ])
    .areas(tabs_and_main);

    if state.adding {
        let [todo_area, input_area] =
            Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).areas(main_area);
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
        let color = Color::from_u32(tag_sys.map()[t].color());
        spans.push(Span::styled(str, color));
    }
    spans
}

fn render_main_screen(frame: &mut Frame, state: &mut State) -> Option<Rect> {
    let (tab_area, todo_area, input_area) = compute_main_layout(frame, state);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::ORANG)
        .bg(theme::BG0)
        .title("Tabs")
        .render(tab_area, frame.buffer_mut());

    let outer_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::BLUE)
        .bg(theme::BG0)
        .title("Todos".to_span().into_centered_line());

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
    .block(outer_block)
    .fg(theme::TEXT)
    .bg(theme::BG0)
    .highlight_style(Style::default().fg(theme::ORANG))
    .highlight_symbol(">");

    frame.render_stateful_widget(list, todo_area, &mut state.list);
    input_area
}

fn render(frame: &mut Frame, state: &mut State) {
    let input_area = render_main_screen(frame, state);

    if state.adding {
        assert!(input_area.is_some(), "input area not ready");
        let area = input_area.unwrap();
        render_input_window(area, state, frame);
        state.anims.set_progress(true, state.input_anim, area);
    } else {
        state.anims.reset_anim(state.input_anim);
        state
            .anims
            .set_progress(false, state.input_anim, Rect::default());
    }

    state.anims.progress(frame, state.dt);
}
