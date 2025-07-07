use crate::theme;
use std::collections::HashMap;
use tachyonfx::fx;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::State;
use tachyonfx::{Duration as FxDuration, Effect, Shader};

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

pub(crate) use add_anim_if_missing;
pub(crate) use after_anim;

#[derive()]
pub struct Animation {
    pub effect: Effect,
    pub area: Rect,
    pub should_progress: bool,
    trigger: Option<Box<dyn Fn(&State, &AnimationHandler) -> bool>>,
    last_trigger: bool,
}

pub struct AnimationHandler {
    pub animations: HashMap<String, Box<Animation>>,
}

impl AnimationHandler {
    pub fn add(
        self: &mut Self,
        name: &str,
        effect: Effect,
        area: Rect,
        trigger: Option<Box<dyn Fn(&State, &AnimationHandler) -> bool>>,
    ) -> usize {
        let anim = Box::new(Animation {
            effect: effect,
            area,
            should_progress: false,
            trigger: trigger,
            last_trigger: false,
        });
        self.animations.insert(name.to_string(), anim);

        self.animations.len() - 1
    }

    pub fn running(self: &Self) -> bool {
        self.animations.values().any(|a| a.should_progress)
    }

    pub fn progress(self: &mut Self, frame: &mut Frame, dt: f64, state: &State) {
        let keys: Vec<(usize, String)> =
            self.animations.keys().cloned().enumerate().collect();
        let mut triggers: Vec<bool> = vec![false; self.animations.len()];

        for (i, k) in keys.iter() {
            let a = self.animations.get(k).unwrap();
            triggers[*i] = if let Some(t) = &a.trigger {
                t(state, self)
            } else {
                false
            };
        }

        for (i, k) in keys {
            let a = self.animations.get_mut(&k).unwrap();
            let b = triggers[i];

            if b {
                if !a.last_trigger {
                    a.effect.reset();
                }
                a.should_progress = true;
            }
            a.last_trigger = b;

            let time = if a.should_progress {
                (dt * 1000.0) as u32
            } else {
                0
            };

            a.effect
                .process(FxDuration::from_millis(time), frame.buffer_mut(), a.area);
        }
    }
}

pub fn render_intro(frame: &mut Frame, state: &mut State) {
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
