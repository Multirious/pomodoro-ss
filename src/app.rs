use std::time::{Duration, SystemTime, SystemTimeError};

use bevy_ecs::prelude::*;

#[derive(Component)]
struct Stopwatch {
    start: SystemTime,
    pub is_paused: bool,
}

fn stopwatch_system(stopwatch: Query<&mut SystemStopwatch>) {}

struct Timer {
    stopwatch: SystemStopwatch,
    duration: Duration,
}

impl SystemTimer {
    fn is_done(&self) -> bool {
        let Ok(elasped) = self.stopwatch.start.elapsed() else {
            return false;
        };
        elasped >= self.duration
    }
}

fn timer_system() {}

fn run() {
    let mut world = World::new();
}
