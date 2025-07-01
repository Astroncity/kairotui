use crate::State;
use ratatui::{Frame, layout::Rect};
use tachyonfx::{Duration as FxDuration, Effect, Shader};
use tracing::{Level, event};

#[derive()]
pub struct Animation {
    effect: Effect,
    pub area: Rect,
    should_progress: bool,
    trigger: Option<Box<dyn Fn(&State) -> bool>>,
    last_trigger: bool,
}

pub struct AnimationHandler {
    pub animations: Vec<Box<Animation>>,
}

impl AnimationHandler {
    pub fn add(self: &mut Self, effect: Effect, area: Rect, trigger: Option<Box<dyn Fn(&State) -> bool>>) -> usize {
        self.animations.push(Box::new(Animation {
            effect: effect,
            area,
            should_progress: false,
            trigger: trigger,
            last_trigger: false,
        }));

        self.animations.len() - 1
    }

    pub fn running(self: &Self) -> bool {
        self.animations.iter().any(|a| a.should_progress)
    }

    pub fn progress(self: &mut Self, frame: &mut Frame, dt: f64, state: &State) {
        event!(Level::INFO, "inside my_function!");
        self.animations.iter_mut().for_each(|a| {
            if let Some(t) = &a.trigger {
                let b = t(state);
                if b {
                    if !a.last_trigger {
                        a.effect.reset();
                    }
                    a.should_progress = true;
                }
                a.last_trigger = b;
            }

            if a.should_progress {
                a.effect.process(
                    FxDuration::from_millis((dt * 1000.0) as u32),
                    frame.buffer_mut(),
                    a.area,
                );
            }
        });
    }

    pub fn reset_anim(self: &mut Self, idx: usize) {
        self.animations[idx].effect.reset();
    }
}
