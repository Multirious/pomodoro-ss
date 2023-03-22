#![allow(unused)]
// #![windows_subsystem = "windows"]

use std::sync::mpsc;
use std::time::{Duration, Instant};

use activity_monitor::{ActivityKind, ActivityMonitor};
use anyhow::{bail, Result};
use device_query::{DeviceQuery, DeviceState};
use time::{Stopwatch, Timer};

mod activity_monitor;
mod break_notifier;
#[cfg(windows)]
mod os;
mod time;

pub struct World {
    delta: Duration,
}

impl World {
    pub fn delta(&self) -> Duration {
        self.delta
    }
}

#[derive(Debug)]
enum TrayInputEvent {
    Quit,
    RestartWork,
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

    let mut last_frame_time = Instant::now();

    loop {
        world.delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        std::thread::sleep_ms(10);

        // Code down here
    }
    Ok(())
}
