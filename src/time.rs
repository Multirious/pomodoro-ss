use std::time::Duration;

use crate::World;

#[derive(Debug, Clone)]
pub struct Stopwatch {
    time: Duration,
    pub pause: bool,
}

impl Stopwatch {
    pub fn start_at(start: Duration) -> Stopwatch {
        Stopwatch {
            time: start,
            pause: false,
        }
    }

    pub fn new() -> Stopwatch {
        Stopwatch {
            time: Duration::ZERO,
            pause: false,
        }
    }

    pub fn time(&self) -> Duration {
        self.time
    }

    pub fn restart(&mut self) {
        self.time = Duration::ZERO;
        self.pause = false;
    }

    pub fn update(&mut self, world: &World) {
        if self.pause {
            return;
        }
        self.time = self.time.saturating_add(world.delta());
    }
}

#[derive(Debug, Clone)]
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

    pub fn update(&mut self, world: &World) {
        if self.pause {
            return;
        }
        self.time_left = self.time_left.saturating_sub(world.delta());
    }
}

#[derive(Debug, Clone)]
struct FormattedDuration {
    hours: u32,
    minutes: u32,
    seconds: u32,
}

impl FormattedDuration {
    pub fn new(duration: Duration) -> FormattedDuration {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds_left = total_seconds % 60;
        let hours = minutes / 60;
        let minutes_left = minutes % 60;
        FormattedDuration {
            hours: hours as u32,
            minutes: minutes_left as u32,
            seconds: seconds_left as u32,
        }
    }
}

impl From<FormattedDuration> for Duration {
    fn from(value: FormattedDuration) -> Self {
        let secs =
            (value.hours as u64 * 60 * 60) + (value.minutes as u64 * 60) + value.seconds as u64;
        Duration::from_secs(secs)
    }
}

impl From<Duration> for FormattedDuration {
    fn from(value: Duration) -> Self {
        FormattedDuration::new(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn formatted_duration() {
        assert_eq!(
            Duration::from_secs(12345),
            FormattedDuration {
                hours: 3,
                minutes: 25,
                seconds: 45,
            }
            .into()
        )
    }
}
