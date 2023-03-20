#![allow(unused)]
// #![windows_subsystem = "windows"]

use std::sync::mpsc;
use std::time::{Duration, Instant};

use activity::{ActivityKind, ActivityMonitor};
use anyhow::{bail, Result};
use device_query::{DeviceQuery, DeviceState};
use time::{Stopwatch, Timer};

mod activity;
mod time;

trait Process {
    fn update(&mut self, world: &mut World);
}

pub struct World {
    delta: Duration,
}

impl World {
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

enum PomodoroState {
    Work,
    Breaking,
}

#[derive(Debug)]
enum TrayInputEvent {
    Quit,
    RestartWork,
}

struct TimeCfg {
    time_break: Duration,
    time_work: Duration,
    time_warn_before_break: Duration,
    time_alert_before_break: Duration,
    time_count_as_idle: Duration,
}

impl TimeCfg {
    pub fn time_loop(&self) -> Duration {
        self.time_work + self.time_break
    }
}

fn main() -> Result<()> {
    let (tray_item_send, tray_item_recv) = mpsc::sync_channel::<u32>(10);
    // let mut tray_item = tray_item()?;
    // let tray_item_send_cloned = tray_item_send.clone();
    // tray_item.add_menu_item("Quit", move || {
    //     let _result = tray_item_send_cloned.try_send(TrayInputEvent::Quit);
    // })?;
    // tray_item.add_menu_item("Restart work", move || {
    //     let _result = tray_item_send.try_send(TrayInputEvent::RestartWork);
    // })?;

    let mut world = World {
        delta: Duration::ZERO,
    };

    let mut activity_monitor = ActivityMonitor::new(
        &world,
        Box::new(|_, activity_kind, amount| match activity_kind {
            ActivityKind::KeyPress => 1.0 * amount as f64,
            ActivityKind::KeyJustPress => 75.0 * amount as f64,
            ActivityKind::MousePressed => 1.0 * amount as f64,
            ActivityKind::MouseJustPressed => 75.0 * amount as f64,
            ActivityKind::MouseMove { pixels } => pixels.magnitude() / 10.0 * amount as f64,
        }),
        4096,
    );

    let time_cfg = TimeCfg {
        time_break: Duration::from_secs(60 * 10),
        time_work: Duration::from_secs(60 * 20),
        time_warn_before_break: Duration::from_secs(60 * 2),
        time_alert_before_break: Duration::from_secs(30),
        time_count_as_idle: Duration::from_secs(30),
    };
    let time_loop = time_cfg.time_loop();

    let activity_level_threshold_to_force_break: f64 = 1000.0;

    let mut pomodoro_state = PomodoroState::Work;

    let mut current_working_time: Timer = Timer::new(time_cfg.time_work);
    let mut when_last_active: Instant = Instant::now();
    let mut current_break_time: Stopwatch = Stopwatch::new(Duration::ZERO);

    let mut last_frame_time = Instant::now();

    loop {
        world.delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        std::thread::sleep_ms(10);

        activity_monitor.update(&mut world);
        let activity_rate = activity_monitor.activity_rate_in_the_last(Duration::from_secs(5));

        println!("{}\n{}", activity_rate.0, activity_rate.1);

        // Events
        // let event = {
        //     let mut events = vec![];

        //     if let Ok(v) = tray_item_recv.try_recv() {
        //         events.push(Event::TrayEvent(v));
        //     }

        //     events
        // };

        // Code

        current_break_time.update(&mut world);
        current_working_time.update(&mut world);
        activity_monitor.update(&mut world);
    }
    Ok(())
}

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
        Duration::from_secs(
            (value.hours as u64 * 60 * 60) + (value.minutes as u64 * 60) + value.seconds as u64,
        )
    }
}

impl From<Duration> for FormattedDuration {
    fn from(value: Duration) -> Self {
        FormattedDuration::new(value)
    }
}

#[test]
fn test_formatted_duration() {
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

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub fn magnitude_squared(&self) -> f64 {
        let Vec2 { x, y } = self;
        x * x + y * y
    }

    pub fn normalized(&self) -> Vec2 {
        let magnitude = self.magnitude();
        let Vec2 { x, y } = self;
        Vec2 {
            x: x / magnitude,
            y: y / magnitude,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from((x, y): (i32, i32)) -> Self {
        Vec2 {
            x: x as f64,
            y: y as f64,
        }
    }
}

fn tray_item() -> Result<tray_item::TrayItem> {
    Ok(tray_item::TrayItem::new("Pomodoro SS", "timer_icon")?)
}

fn bool_to_cbool(b: bool) -> winapi::shared::minwindef::BOOL {
    if b {
        winapi::shared::minwindef::TRUE
    } else {
        winapi::shared::minwindef::FALSE
    }
}

fn block_input(block: bool) -> Result<()> {
    unsafe {
        let result = winapi::um::winuser::BlockInput(bool_to_cbool(block));
        if result == winapi::shared::minwindef::FALSE {
            bail!(std::io::Error::last_os_error())
        }
        Ok(())
    }
}

fn notify(summary: &str, body: &str) -> Result<()> {
    notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .show()?;
    Ok(())
}
