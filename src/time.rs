use std::time::Duration;

use crate::{Process, World};

pub struct Stopwatch {
    time: Duration,
    pub pause: bool,
}

impl Stopwatch {
    pub fn new(time_start: Duration) -> Stopwatch {
        Stopwatch {
            time: time_start,
            pause: false,
        }
    }

    pub fn time(&self) -> Duration {
        self.time
    }
}

impl Process for Stopwatch {
    fn update(&mut self, world: &mut World) {
        if self.pause {
            return;
        }
        self.time = self.time.saturating_add(world.delta());
    }
}

pub struct Timer {
    time_left: Duration,
    pub pause: bool,
}

impl Timer {
    pub fn new(time_start: Duration) -> Timer {
        Timer {
            time_left: time_start,
            pause: false,
        }
    }

    pub fn time_left(&self) -> Duration {
        self.time_left
    }
}

impl Process for Timer {
    fn update(&mut self, world: &mut World) {
        if self.pause {
            return;
        }
        self.time_left = self.time_left.saturating_sub(world.delta());
    }
}
