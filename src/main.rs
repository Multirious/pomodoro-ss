use std::time::{Duration, Instant};

use anyhow::{bail, Result};

enum State {
    FirstHalfWorking,
    LastHalfWorking,
    Breaking,
}

fn main() -> Result<()> {
    const SECS_ONE_MIN: u64 = 60;
    const FIRST_HALF_WORK_TIME: u64 = SECS_ONE_MIN * 25;
    const LAST_HALF_WORK_TIME: u64 = SECS_ONE_MIN * 5;
    const BREAK_TIME: u64 = SECS_ONE_MIN * 10;
    let mut time = Instant::now();
    let mut state = State::FirstHalfWorking;
    block_input(true)?;
    block_input(false)?;
    notify("Started!", "Pomodoro Super Strict Started!")?;
    loop {
        std::thread::sleep(Duration::from_secs(1));
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
