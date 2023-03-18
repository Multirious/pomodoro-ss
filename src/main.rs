#![windows_subsystem = "windows"]

use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};

enum State {
    FirstHalfWorking,
    LastHalfWorking,
    Breaking,
}

#[derive(Debug)]
enum TrayInput {
    Quit,
}

struct Timer {
    time_left: Duration,
    is_pause: bool,
}

impl Timer {
    pub fn new(time: Duration) -> Timer {
        Timer {
            time_left: time,
            is_pause: false,
        }
    }

    pub fn process(&mut self, delta: Duration) {
        self.time_left = self.time_left.saturating_sub(delta);
    }

    pub fn is_end(&self) -> bool {
        self.time_left == Duration::ZERO
    }

    pub fn pause(&mut self, pause: bool) {
        self.is_pause = pause
    }
}

fn main() -> Result<()> {
    const SECS_ONE_MIN: u64 = 60;
    const FIRST_HALF_WORK_TIME: u64 = SECS_ONE_MIN * 20;
    const LAST_HALF_WORK_TIME: u64 = SECS_ONE_MIN * 5;
    const BREAK_TIME: u64 = SECS_ONE_MIN * 10;
    let mut time = Instant::now();
    let mut state = State::FirstHalfWorking;
    let mut tray_item = tray_item()?;
    let (tray_send, tray_recv) = mpsc::sync_channel(4);

    tray_item.add_menu_item("Quit", move || {
        let _res = tray_send.send(TrayInput::Quit);
    })?;
    block_input(true)?;
    block_input(false)?;
    notify("Started!", "Pomodoro Super Strict Started!")?;
    loop {
        std::thread::sleep(Duration::from_micros(500));
        match tray_recv.recv_timeout(Duration::from_micros(100)) {
            Ok(i) => match i {
                TrayInput::Quit => break,
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(e) => bail!(e),
        }
        match state {
            State::FirstHalfWorking => {
                if time.elapsed() < Duration::from_secs(FIRST_HALF_WORK_TIME) {
                    continue;
                };
                notify("Warn", "First half elasped!")?;
                time = Instant::now();
                state = State::LastHalfWorking
            }
            State::LastHalfWorking => {
                if time.elapsed() < Duration::from_secs(LAST_HALF_WORK_TIME) {
                    continue;
                };
                notify("Break", "Break time!")?;
                time = Instant::now();
                state = State::Breaking;
            }
            State::Breaking => {
                if time.elapsed() < Duration::from_secs(BREAK_TIME) {
                    let _res = block_input(true);
                    continue;
                };
                let _res = block_input(false);
                let _res = block_input(false);
                let _res = block_input(false);
                notify("Work", "Back to work!")?;
                time = Instant::now();
                state = State::FirstHalfWorking;
            }
        }
    }
    Ok(())
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
