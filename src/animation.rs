use crate::theme;
use ratatui::{Frame, layout::Rect};
use tachyonfx::{Duration as FxDuration, Effect, Motion, Shader, fx::sweep_in};

#[derive(Debug)]
pub struct Animation {
    effect: Effect,
    area: Rect,
    should_progress: bool,
}

#[derive(Debug, Default)]
pub struct AnimationHandler {
    animations: Vec<Box<Animation>>,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            effect: sweep_in(
                Motion::LeftToRight,
                16,
                0,
                theme::BG0,
                FxDuration::default(),
            ),
            area: Rect::default(),
            should_progress: false,
        }
    }
}

impl AnimationHandler {
    pub fn add(self: &mut Self, effect: Effect, area: Rect) -> usize {
        self.animations.push(Box::new(Animation {
            effect: effect,
            area,
            should_progress: false,
        }));

        self.animations.len() - 1
    }

    pub fn running(self: &Self) -> bool {
        self.animations.iter().any(|a| a.should_progress)
    }

    pub fn progress(self: &mut Self, frame: &mut Frame, dt: f64) {
        self.animations.iter_mut().for_each(|a| {
            if a.should_progress {
                a.effect.process(
                    FxDuration::from_millis((dt * 1000.0) as u32),
                    frame.buffer_mut(),
                    a.area,
                );
            }
        });
    }

    pub fn set_progress(self: &mut Self, flag: bool, idx: usize, area: Rect) {
        assert!(idx < self.animations.len());
        self.animations[idx].should_progress = flag;
        self.animations[idx].area = area;
    }

    pub fn reset_anim(self: &mut Self, idx: usize) {
        self.animations[idx].effect.reset();
    }
}
