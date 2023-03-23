#![allow(unused)]
// #![windows_subsystem = "windows"]

use std::ops::ControlFlow;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use activity_monitor::{ActivityKind, ActivityMonitor};
use anyhow::{bail, Error, Result};
use break_notifier::BreakState;
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

fn init_tray_item() -> Result<mpsc::Receiver<TrayInputEvent>> {
    let mut tray_item = os::tray_item()?;
    let (tray_item_send, tray_item_recv) = mpsc::sync_channel(10);
    let tray_item_send_cloned = tray_item_send.clone();
    tray_item.add_menu_item("Quit", move || {
        let _result = tray_item_send_cloned.try_send(TrayInputEvent::Quit);
    })?;
    Ok(tray_item_recv)
}

fn main() -> Result<()> {
    let tray_item_recv = init_tray_item()?;

    let mut break_notifier = break_notifier::BasicTimeBreak::new(
        BreakState::NotBreak,
        Duration::from_secs(10),
        Duration::from_secs(10),
    );
    break_notifier.set_start_break_callback(Some(|| {}));
    break_notifier.set_end_break_callback(Some(|| {}));

    main_loop_run(|world| {
        match tray_item_recv.try_recv() {
            Ok(o) => match o {
                TrayInputEvent::Quit => return ControlFlow::Break(Ok(())),
                TrayInputEvent::RestartWork => todo!(),
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(e) => return ControlFlow::Break(Err(e)),
        }
        break_notifier.update(world);
        ControlFlow::Continue(())
    })?;

    Ok(())
}

pub enum Void {}

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
