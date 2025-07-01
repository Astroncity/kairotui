use std::collections::HashMap;

use crate::State;
use ratatui::{Frame, layout::Rect};
use tachyonfx::{Duration as FxDuration, Effect, Shader};
use tracing::{Level, event};

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
        let keys: Vec<(usize, String)> = self.animations.keys().cloned().enumerate().collect();
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

            let time = if a.should_progress { (dt * 1000.0) as u32 } else { 0 };

            a.effect
                .process(FxDuration::from_millis(time), frame.buffer_mut(), a.area);
        }
    }
}
