use std::time::Duration;

use crate::{
    activity_monitor::{ActivityKind, ActivityMonitor},
    time::{Stopwatch, Timer},
    Process, World,
};

pub enum BreakState {
    Break,
    NotBreak,
}

pub struct NotifyTime {
    break_duration: Duration,
    break_timer: Stopwatch,
    not_break_duration: Duration,
    not_break_timer: Stopwatch,
    state: BreakState,
}

impl NotifyTime {
    pub fn new(
        in_state: BreakState,
        break_duration: Duration,
        not_break_duration: Duration,
    ) -> NotifyTime {
        NotifyTime {
            break_duration,
            not_break_duration,
            not_break_timer: Stopwatch::new(),
            break_timer: Stopwatch::new(),
            state: in_state,
        }
    }
}

pub struct NotifyActivity {
    activity_monitor: ActivityMonitor,

    high_activity_level: f64,
    consecutive_high_activity_level_duration: Duration,
    current_consecutive_high_acticity_level: Stopwatch,

    break_duration: Duration,
    duration_count_as_idle: Duration,
    current_idle_duration: Stopwatch,

    state: BreakState,
}

impl NotifyActivity {
    pub fn new(
        in_state: BreakState,
        high_activity_level: f64,
        threshold: f64,
        duration: Duration,
        duration_count_as_idle: Duration,
        break_duration: Duration,
    ) -> NotifyActivity {
        NotifyActivity {
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
