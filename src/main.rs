mod animation;
mod data;
mod log;
mod tab;
mod tag;
mod theme;

use animation::{AnimationHandler, add_anim_if_missing, after_anim};

#[allow(unused_imports)]
use tracing::{info, warn};

use crate::{data::PersistentData, log::LogList, tab::Tab, tag::*};
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
use std::{cell::RefCell, fs, rc::Rc};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tachyonfx::{
    Duration as FxDuration, Shader,
    fx::{self},
};
use tracing_subscriber::FmtSubscriber;

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
    rendered_lists: Vec<Box<dyn Tab>>,
    dt: f64,
}

impl State {
    pub fn update_input_display(&mut self) {
        let regex = Regex::new(r"(tag:\s(\w+))+$").unwrap();

        self.input_display = if regex.is_match(&self.input) {
            let index = self.input.find(" tag: ").unwrap();

            Line::from(vec![
                Span::styled(self.input[..index].to_string(), theme::TEXT),
                Span::styled(self.input[index..].to_string(), theme::ORANGE),
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
        *l = Span::styled(format!("{}█", l.content.clone()), l.style);
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
                .fg(theme::ORANGE)
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
        focused_list: tab::ListType::Log,
        focused_list_idx: 0,
        popup_active: false,
        popup_msg: Span::raw(""),
        input_default: ("", ""),
        rendered_lists: vec![],
        dt: 0.0,
    };

    state.main_panel_title = tab::ListType::Log.to_str();
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

    // Use Rc<RefCell<State>> separately and clone it into LogList
    let state_rc = Rc::new(RefCell::new(state));

    // Push a reference-cloned version into rendered_lists
    let log_list = Box::new(LogList::new("Logs", state_rc.clone()));

    // Now mutate the actual state to insert the list
    state_rc.borrow_mut().rendered_lists.push(log_list);

    Rc::try_unwrap(state_rc)
        .map(|rc| rc.into_inner())
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Rc — multiple references exist")) // Return the Rc unwrapped (if needed) or keep it Rc throughout
}

fn delegate_enter(state: &mut State) {
    match state.focused_list {
        tab::ListType::Log => {
            if let Some(i) = state.list_state.selected() {
                let log = &mut state.data.logs[i];
                log.done = true;
                state.data.past_logs.push(log.clone());
                state.data.logs.remove(i);
            }
        }
        tab::ListType::Tag => {
            state.input_dialog_active = true;
            state.input_default.0 = " Edit Tag ";
            state.input_default.1 =
                "<name>: <hex> (e.g. #FF00FF) or <color name> (e.g. Green)";
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
                if state.focused_list == tab::ListType::Log {
                    state.input_dialog_active = true;
                    state.input_default.0 = " New Log ";
                    state.input_default.1 = "<log_name> (tag: <tag_name>)*";
                }
            }
            'D' => match state.focused_list {
                tab::ListType::Log => {
                    log::delete_selected(state);
                }
                tab::ListType::PastLog => {
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
            'J' => ch_tab(state, true),
            'K' => ch_tab(state, false),
            _ => {}
        },
        event::KeyCode::Enter => {
            delegate_enter(state);
        }
        event::KeyCode::Tab => ch_tab(state, true),
        _ => {}
    }
    false
}

fn ch_tab(state: &mut State, down: bool) {
    let ch = if down { 1 } else { -1 };
    let prev = state.focused_list_idx;
    let len = tab::ListType::TYPES.len();
    state.focused_list_idx = prev.checked_add_signed(ch as isize).unwrap_or(len - 1);
    state.focused_list_idx %= len;
    state.focused_list = tab::ListType::TYPES[state.focused_list_idx];
    state.list_state.scroll_up_by(u16::MAX);
    state.main_panel_title = state.focused_list.to_str();
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
            tab::ListType::Log => {
                if let Some(str) = res.0 {
                    log::handle_add(state, str);
                }
            }
            tab::ListType::Tag => {
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
        // tab::ListType::Log => {
        //     log::render_log_list(state, &outer, &log_a, frame, log::LogType::Active);
        // }
        tab::ListType::PastLog => {
            log::render_log_list(state, &outer, &log_a, frame, log::LogType::Past);
        }
        tab::ListType::Tag => render_tag_list(state, &outer, &log_a, frame),
        _ => {}
    }

    // WARN:
    state.rendered_lists[0].render(&outer, &log_a, frame);

    tab::render_tab_list(&tab_area, state, frame);
}

fn render(frame: &mut Frame, state: &mut State) {
    if state.data.opened_once
        || state.anims.borrow().animations.contains_key("intro_end")
            && state.anims.borrow().animations["intro_end"].effect.done()
    {
        render_main_screen(frame, state);
    } else {
        animation::render_intro(frame, state);
    }

    if state.input_dialog_active {
        render_input_dialog(state.input_default.0, state.input_default.1, frame, state);
    }

    if state.popup_active {
        render_popup(" Popup ", &state.popup_msg, frame);
    }

    state.anims.borrow_mut().progress(frame, state.dt, state);
}
