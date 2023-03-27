use std::sync::mpsc;
use std::time::Duration;

use crate::{
    activity_monitor::{ActivityKind, ActivityMonitor},
    time::{Stopwatch, Timer},
    World,
};

#[derive(Debug, Clone, Copy)]
pub enum BreakState {
    Break,
    NotBreak,
}

pub struct BasicTimeBreak {
    break_duration: Duration,
    break_timer: Stopwatch,
    not_break_duration: Duration,
    not_break_timer: Stopwatch,
    state: BreakState,

    start_break_callback: Option<Box<dyn Fn()>>,
    end_break_callback: Option<Box<dyn Fn()>>,
}

impl BasicTimeBreak {
    pub fn new(
        in_state: BreakState,
        break_duration: Duration,
        not_break_duration: Duration,
    ) -> BasicTimeBreak {
        BasicTimeBreak {
            break_duration,
            not_break_duration,
            not_break_timer: Stopwatch::new(),
            break_timer: Stopwatch::new(),
            state: in_state,

            start_break_callback: None,
            end_break_callback: None,
        }
    }

    pub fn time_before_start_break(&self) -> Option<Duration> {
        if self.not_break_timer.pause {
            None
        } else {
            Some(self.not_break_duration - self.not_break_timer.time())
        }
    }

    pub fn time_before_end_break(&self) -> Option<Duration> {
        if self.break_timer.pause {
            None
        } else {
            Some(self.break_duration - self.break_timer.time())
        }
    }

    pub fn advance_timer(&mut self, by: Duration) {
        match self.state {
            BreakState::Break => self.break_timer.advance(by),
            BreakState::NotBreak => self.not_break_timer.advance(by),
        }
    }

    /// This will not call the callback
    pub fn switch_to(&mut self, state: BreakState) {
        self.state = state;
        match state {
            BreakState::Break => {
                self.not_break_timer.pause = true;
                self.break_timer.restart();
            }
            BreakState::NotBreak => {
                self.break_timer.pause = true;
                self.not_break_timer.restart();
            }
        }
    }

    pub fn update(&mut self, world: &World) {
        match self.state {
            BreakState::Break => {
                self.break_timer.update(world);
                if self.break_timer.time() > self.break_duration {
                    self.break_timer.pause = true;
                    if let Some(f) = self.end_break_callback.as_ref() {
                        f()
                    }
                    self.not_break_timer.restart();
                    self.state = BreakState::NotBreak;
                }
            }
            BreakState::NotBreak => {
                self.not_break_timer.update(world);
                if self.not_break_timer.time() > self.not_break_duration {
                    self.not_break_timer.pause = true;
                    if let Some(f) = self.start_break_callback.as_ref() {
                        f()
                    }
                    self.break_timer.restart();
                    self.state = BreakState::Break;
                }
            }
        }
    }

    pub fn break_state(&self) -> BreakState {
        self.state
    }

    pub fn set_start_break_callback<F>(&mut self, f: Option<F>)
    where
        F: Fn() + 'static,
    {
        self.start_break_callback = f.map(|f| Box::new(f) as _)
    }

    pub fn set_end_break_callback<F>(&mut self, f: Option<F>)
    where
        F: Fn() + 'static,
    {
        self.end_break_callback = f.map(|f| Box::new(f) as _)
    }
}

pub struct ActivityBreak {
    activity_monitor: ActivityMonitor,

    high_activity_level: f64,
    consecutive_high_activity_level_duration: Duration,
    current_consecutive_high_acticity_level: Stopwatch,

    break_duration: Duration,
    duration_count_as_idle: Duration,
    current_idle_duration: Stopwatch,

    state: BreakState,
}

impl ActivityBreak {
    pub fn new(
        in_state: BreakState,
        high_activity_level: f64,
        threshold: f64,
        duration: Duration,
        duration_count_as_idle: Duration,
        break_duration: Duration,
    ) -> ActivityBreak {
        ActivityBreak {
            activity_monitor: ActivityMonitor::new(
                |_, activity_kind, amount| match activity_kind {
                    ActivityKind::KeyPress => 1.0 * amount as f64,
                    ActivityKind::KeyJustPress => 75.0 * amount as f64,
                    ActivityKind::MousePressed => 1.0 * amount as f64,
                    ActivityKind::MouseJustPressed => 75.0 * amount as f64,
                    ActivityKind::MouseMove { distance } => distance / 10.0 * amount as f64,
                },
                4096,
            ),
            high_activity_level,
            consecutive_high_activity_level_duration: duration,
            current_consecutive_high_acticity_level: Stopwatch::new(),
            break_duration,
            duration_count_as_idle,
            current_idle_duration: Stopwatch::new(),
            state: in_state,
        }
    }
}
