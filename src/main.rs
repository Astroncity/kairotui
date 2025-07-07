mod animation;
mod log;
mod tab;
mod tag;
mod theme;

use crate::log::Log;

use animation::AnimationHandler;

#[allow(unused_imports)]
use tracing::{info, warn};

use crate::tag::*;
use anyhow::{Ok, Result};
use dirs::config_dir;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent},
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    text::{Line, Span, ToSpan},
    widgets::{Block, BorderType, Clear, ListState, Paragraph, Widget},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, fs};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tachyonfx::{
    Duration as FxDuration, Shader,
    fx::{self},
};
use tracing_appender;
use tracing_subscriber::FmtSubscriber;

macro_rules! add_anim_if_missing {
    ($state:expr, $key:expr, $effect:expr, $area:expr, $trigger:expr) => {
        if !$state.anims.borrow().animations.contains_key($key) {
            $state.anims.borrow_mut().add(
                $key,
                $effect,
                $area,
                Some(Box::new($trigger)),
            );
        }
    };
}

macro_rules! after_anim {
    ($anim_handler:expr, $anim:expr) => {
        $anim_handler.animations.get($anim).unwrap().effect.done()
    };
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct PersistentData {
    opened_once: bool,
    logs: Vec<Log>,
    past_logs: Vec<Log>,
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

struct State {
    data: PersistentData,
    list_state: ListState,
    input_dialog_active: bool,
    popup_active: bool,
    popup_msg: Span<'static>,
    input: String,
    input_default: (&'static str, &'static str),
    input_display: Line<'static>,
    main_panel_title: &'static str,
    anims: RefCell<AnimationHandler>,
    focused_list_idx: usize,
    focused_list: tab::ListType,
    dt: f64,
}

impl State {
    pub fn update_input_display(&mut self) {
        let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();

        self.input_display = if regex.is_match(&self.input) {
            let index = self.input.find(" tag: ").unwrap();

            Line::from(vec![
                Span::styled(self.input[..index].to_string(), theme::TEXT),
                Span::styled(self.input[index..].to_string(), theme::ORANG),
            ])
        } else {
            Line::from(vec![Span::raw(self.input.clone())])
        };
    }
}

fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::daily(
        "/home/astro/projects/kairotui/logs",
        "kairotui.log",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut state = init()?;

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();
    result?;
    Ok(())
}

fn render_popup(title: &str, msg: &Span, frame: &mut Frame) {
    let c = msg.style.fg.unwrap_or(theme::TEXT);
    let area = {
        let vert = Layout::vertical([Constraint::Percentage(15)]).flex(Flex::Center);
        let horz = Layout::horizontal([Constraint::Percentage(30)]).flex(Flex::Center);
        let [area] = vert.areas(frame.area());
        let [area] = horz.areas(area);
        area
    };
    Paragraph::new(msg.clone())
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(c)
                .title(title.to_span().into_centered_line()),
        )
        .centered()
        .render(area, frame.buffer_mut());
}

fn render_input_dialog(title: &str, def: &str, frame: &mut Frame, state: &mut State) {
    let has_input = !state.input.is_empty();
    let txt = if !has_input {
        Line::from(def)
    } else {
        let mut s = state.input_display.clone();
        let l: &mut Span = s.spans.iter_mut().last().unwrap();
        *l = Span::styled(format!("{}█", l.content.clone().to_string()), l.style);
        s
    };

    let color = if has_input {
        theme::TEXT
    } else {
        theme::TEXT_ALT
    };

    let area = {
        let vert = Layout::vertical([Constraint::Percentage(8)]).flex(Flex::Center);
        let horz = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);
        let [area] = vert.areas(frame.area());
        let [area] = horz.areas(area);
        area
    };

    frame.render_widget(Clear, area);
    Paragraph::new(txt.clone())
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(theme::ORANG)
                .title(title.to_span().into_centered_line()),
        )
        .left_aligned()
        .fg(color)
        .render(area, frame.buffer_mut());
}

fn init() -> Result<State> {
    let mut state = State {
        data: PersistentData::default(),
        list_state: ListState::default(),
        input: String::from(""),
        input_dialog_active: false,
        input_display: Line::default(),
        main_panel_title: "",
        anims: RefCell::new(AnimationHandler {
            animations: HashMap::new(),
        }),
        focused_list: tab::ListType::LOG,
        focused_list_idx: 0,
        popup_active: false,
        popup_msg: Span::raw(""),
        input_default: ("", ""),
        dt: 0.0,
    };

    state.main_panel_title = tab::ListType::LOG.to_str();
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
    match state.focused_list {
        tab::ListType::LOG => {
            if let Some(i) = state.list_state.selected() {
                let log = &mut state.data.logs[i];
                log.done = true;
                state.data.past_logs.push(log.clone());
                state.data.logs.remove(i);
            }
        }
        tab::ListType::TAG => {
            state.input_dialog_active = true;
            state.input_default.0 = " Edit Tag ";
            state.input_default.1 = "<name>: <hex_color>";
        }
        _ => {}
    }
}

fn handle_key(key: KeyEvent, state: &mut State) -> bool {
    if state.popup_active {
        state.popup_active = false;
        return false;
    }

    match key.code {
        event::KeyCode::Char(char) => match char {
            'q' => return true,
            'A' => {
                if state.focused_list == tab::ListType::LOG {
                    state.input_dialog_active = true;
                    state.input_default.0 = " New Log ";
                    state.input_default.1 = "<log_name> (tag: <tag_name>)*";
                }
            }
            'D' => match state.focused_list {
                tab::ListType::LOG => {
                    log::delete_selected(state);
                }
                tab::ListType::PASTLOG => {
                    log::delete_past_log(state);
                }
                _ => {}
            },
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
            state.focused_list_idx += 1;
            state.focused_list_idx %= tab::ListType::TYPES.len();
            state.focused_list = tab::ListType::TYPES[state.focused_list_idx];
            state.list_state.scroll_up_by(u16::MAX);
            state.main_panel_title = state.focused_list.to_str();
        }
        _ => {}
    }
    false
}

fn handle_input(key: KeyEvent, state: &mut State) -> (Option<String>, bool) {
    match key.code {
        event::KeyCode::Enter => {
            let ret = Some(state.input.clone());
            state.input.clear();
            state.update_input_display();
            return (ret, true);
        }
        event::KeyCode::Esc => return (None, true),
        event::KeyCode::Backspace => {
            state.input.pop();
        }
        event::KeyCode::Char(c) => {
            state.input.push(c);
        }
        _ => {}
    }
    state.update_input_display();
    (None, false)
}

fn handle_event(state: &mut State) -> bool {
    let _ = state.data.save();
    if let Event::Key(key) = event::read().unwrap() {
        if !state.input_dialog_active {
            return handle_key(key, state);
        }
        let res = handle_input(key, state);
        match state.focused_list {
            tab::ListType::LOG => {
                if let Some(str) = res.0 {
                    log::handle_add(state, str);
                }
            }
            tab::ListType::TAG => {
                if let Some(str) = res.0 {
                    tag::handle_edit(state, str);
                }
            }
            _ => {}
        }
        state.input_dialog_active = !res.1;
    }
    false
}

fn run(mut terminal: DefaultTerminal, state: &mut State) -> Result<()> {
    let mut last_frame = std::time::Instant::now();

    loop {
        let now = Instant::now();
        state.dt = now.duration_since(last_frame).as_secs_f64();
        last_frame = now;

        log::update_logs(&mut state.data.logs);
        terminal.draw(|x| render(x, state))?;

        let timeout = if state.anims.borrow().running() {
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

fn handle_main_layout_anims(areas: &[Rect; 2], state: &mut State) {
    add_anim_if_missing!(
        state,
        "main_area",
        fx::coalesce(FxDuration::from_millis(500)),
        areas[0],
        |s, a| s.data.opened_once || after_anim!(a, "intro_end")
    );
    add_anim_if_missing!(
        state,
        "tab_area",
        fx::coalesce(FxDuration::from_millis(500)),
        areas[1],
        |_, a| after_anim!(a, "main_area")
    );
}

fn compute_main_layout(frame: &Frame, state: &mut State) -> (Rect, Rect) {
    state.data.opened_once = true;
    let [tabs_and_main] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());
    let [tab_area, main_area] =
        Layout::horizontal([Constraint::Length(20), Constraint::Min(10)])
            .areas(tabs_and_main);
    let [todo_area] = Layout::vertical([Constraint::Fill(1)]).areas(main_area);

    handle_main_layout_anims(&[todo_area, tab_area], state);
    (tab_area, todo_area)
}

fn render_main_screen(frame: &mut Frame, state: &mut State) {
    let (tab_area, log_a) = compute_main_layout(frame, state);
    let panel_txt = state.main_panel_title;

    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::BLUE)
        .bg(theme::BG0)
        .title(panel_txt.to_span().into_centered_line());

    match state.focused_list {
        tab::ListType::LOG => {
            log::render_log_list(state, &outer, &log_a, frame, log::LogType::ACTIVE);
        }
        tab::ListType::PASTLOG => {
            log::render_log_list(state, &outer, &log_a, frame, log::LogType::PAST);
        }
        tab::ListType::TAG => render_tag_list(state, &outer, &log_a, frame),
    }

    tab::render_tab_list(&tab_area, state, frame);
}

fn render_intro(frame: &mut Frame, state: &mut State) {
    if state.data.opened_once {
        return;
    }
    let logo = r"
     /$$   /$$           /$$                       /$$               /$$
    | $$  /$$/          |__/                      | $$              |__/
    | $$ /$$/   /$$$$$$  /$$  /$$$$$$   /$$$$$$  /$$$$$$   /$$   /$$ /$$
    | $$$$$/   |____  $$| $$ /$$__  $$ /$$__  $$|_  $$_/  | $$  | $$| $$
    | $$  $$    /$$$$$$$| $$| $$  \__/| $$  \ $$  | $$    | $$  | $$| $$
    | $$\  $$  /$$__  $$| $$| $$      | $$  | $$  | $$ /$$| $$  | $$| $$
    | $$ \  $$|  $$$$$$$| $$| $$      |  $$$$$$/  |  $$$$/|  $$$$$$/| $$
    |__/  \__/ \_______/|__/|__/       \______/    \___/   \______/ |__/
    ";

    let [area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());
    let [inner] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(area);

    Block::bordered()
        .border_type(BorderType::Rounded)
        .fg(theme::BLUE)
        .render(area, frame.buffer_mut());

    Paragraph::new(logo)
        .centered()
        .fg(theme::BLUE)
        .render(inner, frame.buffer_mut());

    let dur = 500;

    add_anim_if_missing!(
        state,
        "intro_start",
        fx::sweep_in(
            tachyonfx::Motion::UpToDown,
            10,
            1,
            theme::BG0,
            FxDuration::from_millis(dur)
        ),
        area,
        |_, _| { true }
    );
    add_anim_if_missing!(
        state,
        "para",
        fx::coalesce(FxDuration::from_millis(dur)),
        inner,
        |_, a| { after_anim!(a, "intro_start") }
    );
    add_anim_if_missing!(
        state,
        "intro_end",
        fx::delay(
            FxDuration::from_millis(dur * 2),
            fx::dissolve(FxDuration::from_millis(dur))
        ),
        area,
        |_, a| after_anim!(a, "para")
    );
}

fn render(frame: &mut Frame, state: &mut State) {
    if state.data.opened_once
        || state.anims.borrow().animations.contains_key("intro_end")
            && state.anims.borrow().animations["intro_end"].effect.done()
    {
        render_main_screen(frame, state);
    } else {
        render_intro(frame, state);
    }

    if state.input_dialog_active {
        render_input_dialog(state.input_default.0, state.input_default.1, frame, state);
    }

    if state.popup_active {
        render_popup(" Popup ", &state.popup_msg, frame);
    }

    state.anims.borrow_mut().progress(frame, state.dt, state);
}
