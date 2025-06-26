mod theme;

use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::ToSpan,
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Widget},
};
use tachyonfx::{Duration, Effect, Shader, fx::sweep_in};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<TodoItem>,
    list_state: ListState,
    adding: bool,
    input: String,
    test_effect: Option<Effect>,
    dt: f64,
}

#[derive(Debug, Default)]
struct TodoItem {
    done: bool,
    description: String,
}

impl TodoItem {
    pub fn new(desc: String) -> Self {
        Self {
            done: false,
            description: desc,
        }
    }
}

fn main() -> Result<()> {
    let mut state = AppState::default();
    state.test_effect = Some(sweep_in(
        tachyonfx::Motion::LeftToRight,
        16,
        0,
        theme::BG0,
        Duration::from_millis(500),
    ));
    color_eyre::install()?;

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
            app.items.push(TodoItem::new(app.input.clone()));
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
        let now = std::time::Instant::now();
        app_state.dt = now.duration_since(last_frame).as_secs_f64();
        last_frame = now;

        terminal.draw(|x| render(x, app_state))?;

        if event::poll(std::time::Duration::from_millis(32))? {
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
    Paragraph::new(app_state.input.as_str())
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
        Duration::from_millis((app_state.dt * 1000.0) as u32),
        frame.buffer_mut(),
        area,
    );
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

    // outer_block.render(todo_area, frame.buffer_mut());

    let list = List::new(app_state.items.iter().map(|e| {
        let v = if e.done {
            e.description.to_span().crossed_out()
        } else {
            e.description.to_span()
        };
        ListItem::from(v)
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
