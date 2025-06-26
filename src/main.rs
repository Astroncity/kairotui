mod theme;

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
use tachyonfx::{Duration as FxDuration, Effect, Shader, fx::sweep_in};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Log>,
    list_state: ListState,
    adding: bool,
    input: String,
    input_display: Line<'static>,
    test_effect: Option<Effect>,
    dt: f64,
}

impl AppState {
    pub const SAVE_PATH: &'static str = "save.dat";

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
    title: String,
    display_text: String,
    #[serde(with = "serde_millis")]
    start: Instant,
    #[serde(with = "serde_millis")]
    end: Instant,
}

impl Log {
    pub fn new(desc: String) -> Self {
        Self {
            display_text: desc.clone(),
            start: Instant::now(),
            end: Instant::now(),
            done: false,
            title: desc,
        }
    }
}

fn save_logs(app_state: &AppState, filename: &str) -> Result<()> {
    let data = serde_json::to_string(&app_state.items).unwrap();
    fs::write(filename, data)?;
    Ok(())
}

fn load(filename: &str) -> Result<Vec<Log>> {
    let str: String = fs::read_to_string(filename)?;
    let vec: Vec<Log> = serde_json::from_str(&str)?;

    Ok(vec)
}

fn main() -> Result<()> {
    let mut state = AppState::default();
    state.test_effect = Some(sweep_in(
        tachyonfx::Motion::LeftToRight,
        16,
        0,
        theme::BG0,
        FxDuration::from_millis(500),
    ));
    color_eyre::install()?;

    if fs::exists(AppState::SAVE_PATH)? {
        state.items = load(AppState::SAVE_PATH).unwrap();
    }

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();
    result
}

fn delete_entry(app: &mut AppState) {
    if let Some(i) = app.list_state.selected() {
        app.items.remove(i);
    }
}

fn handle_add(key: KeyEvent, app: &mut AppState) -> bool {
    match key.code {
        event::KeyCode::Enter => {
            app.items.push(Log::new(app.input.clone()));
            app.input.clear();
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

fn handle_key(key: KeyEvent, app: &mut AppState) -> bool {
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
                app.list_state.select_next();
            }
            'm' => {
                app.list_state.select_previous();
            }
            _ => {}
        },
        event::KeyCode::Enter => {
            if let Some(i) = app.list_state.selected() {
                app.items[i].done = !app.items[i].done;
            }
        }
        _ => {}
    }
    false
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    let mut last_frame = std::time::Instant::now();

    loop {
        let now = Instant::now();
        app_state.dt = now.duration_since(last_frame).as_secs_f64();
        last_frame = now;

        update_logs(&mut app_state.items);
        terminal.draw(|x| render(x, app_state))?;

        if event::poll(Duration::from_millis(32))? {
            let _ = save_logs(app_state, AppState::SAVE_PATH);
            if let Event::Key(key) = event::read()? {
                if app_state.adding {
                    if handle_add(key, app_state) {
                        app_state.adding = false;
                    }
                } else if handle_key(key, app_state) {
                    break;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}

fn compute_main_layout(frame: &Frame, app_state: &AppState) -> (Rect, Rect, Option<Rect>) {
    let [tabs_and_main] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [tab_area, main_area] = Layout::horizontal([
        Constraint::Length(20), // Width of tab area
        Constraint::Min(10),
    ])
    .areas(tabs_and_main);

    if app_state.adding {
        let [todo_area, input_area] =
            Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).areas(main_area);
        (tab_area, todo_area, Some(input_area))
    } else {
        let [todo_area] = Layout::vertical([Constraint::Fill(1)]).areas(main_area);
        (tab_area, todo_area, None)
    }
}

fn render_input_window(area: Rect, app_state: &mut AppState, frame: &mut Frame) {
    Paragraph::new(app_state.input_display.clone())
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(theme::ORANG)
                .bg(theme::BG0)
                .title(" New Task "),
        )
        .fg(theme::TEXT)
        .render(area, frame.buffer_mut());

    app_state.test_effect.as_mut().unwrap().process(
        FxDuration::from_millis((app_state.dt * 1000.0) as u32),
        frame.buffer_mut(),
        area,
    );
}

fn duration_as_hhmmss(dur: Duration) -> String {
    let total_seconds = dur.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn update_logs(logs: &mut [Log]) {
    for log in logs.iter_mut().filter(|l| !l.done) {
        log.end = Instant::now();
        let dur = log.end.duration_since(log.start);
        log.display_text = format!("{} {}", log.title, duration_as_hhmmss(dur));
    }
}

fn render_main_screen(frame: &mut Frame, app_state: &mut AppState) -> Option<Rect> {
    let (tab_area, todo_area, input_area) = compute_main_layout(frame, app_state);

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

    let list = List::new(app_state.items.iter().enumerate().map(|(i, e)| {
        let v = if e.done {
            e.display_text.to_span().crossed_out()
        } else {
            e.display_text.to_span()
        };
        let color = if i % 2 == 0 { theme::BG0 } else { theme::BG1 };
        ListItem::from(v).bg(color)
    }))
    .block(outer_block)
    .fg(theme::TEXT)
    .bg(theme::BG0)
    .highlight_style(Style::default().fg(theme::ORANG))
    .highlight_symbol(">");

    frame.render_stateful_widget(list, todo_area, &mut app_state.list_state);
    input_area
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let input_area = render_main_screen(frame, app_state);

    if app_state.adding {
        assert!(input_area.is_some(), "input area not ready");
        render_input_window(input_area.unwrap(), app_state, frame);
    } else {
        app_state.test_effect.as_mut().unwrap().reset();
    }
}
