#![allow(unused)]
// #![windows_subsystem = "windows"]

use std::{
    ops::ControlFlow,
    sync::mpsc,
    time::{Duration, Instant},
};

use anyhow::{bail, Error, Result};
use device_query::{DeviceQuery, DeviceState};

use activity_monitor::{ActivityKind, ActivityMonitor};
use break_notifier::BreakState;
use time::{Stopwatch, Timer};
use tray_icon::{TrayInputEvent, TrayItem, TrayItemMode};
use utils::*;

mod activity_monitor;
mod break_notifier;
mod notification;
mod os;
mod schedule;
mod time;
mod tray_icon;
mod utils;

pub struct World {
    delta: Duration,
}

impl World {
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

#[derive(Debug)]
enum AppState {
    Break,
    NotBreak,
    GettingUserBreakPreference,
}

fn main() -> Result<()> {
    let (tray_item_sender, tray_item_receiver) = mpsc::sync_channel(10);
    let tray_item = TrayItem::new_with_sender(TrayItemMode::default(), &tray_item_sender)?;

    let mut break_notifier = break_notifier::BasicTimeBreak::new(
        BreakState::NotBreak,
        Duration::from_secs(5),
        Duration::from_secs(5),
    );

    let (break_send, break_recv) = mpsc::sync_channel(2);
    let mut state = AppState::NotBreak;

    let break_send_c = break_send.clone();
    break_notifier.set_start_break_callback(Some(move || break_send_c.just_send(true)));
    break_notifier.set_end_break_callback(Some(move || break_send.just_send(false)));

    main_loop_run(|world| {
        break_notifier.update(world);

        if let Some(tray_input_event) = tray_item_receiver.maybe_recv().break_res_err()? {
            match tray_input_event {
                TrayInputEvent::Quit => return ControlFlow::Break(Ok(())),
                TrayInputEvent::RestartWork => {
                    break_notifier.switch_to(BreakState::NotBreak);
                    state = AppState::NotBreak;
                }
                TrayInputEvent::SkipWork { by } => {
                    if let BreakState::NotBreak = break_notifier.break_state() {
                        break_notifier.advance_timer(by)
                    }
                }
            }
        }
        if let Some(recv_is_break) = break_recv.maybe_recv().break_res_err()? {
            if recv_is_break {
                state = AppState::GettingUserBreakPreference;
            }
        }
        let _res = block_input(state);
        ControlFlow::Continue(())
    })?;

    Ok(())
}

pub fn main_loop_run<F, B>(mut f: F) -> B
where
    F: FnMut(&World) -> ControlFlow<B, ()>,
{
    let mut world = World {
        delta: Duration::ZERO,
    };
    let mut last_frame_time = Instant::now();
    loop {
        world.delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        if let ControlFlow::Break(b) = f(&world) {
            break b;
        };
    }
}

pub fn block_input(block: bool) -> std::result::Result<(), windows::core::Error> {
    unsafe { windows::Win32::UI::Input::KeyboardAndMouse::BlockInput(block).ok() }
}
