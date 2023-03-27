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
mod schedule;
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

fn init_tray_item() -> Result<(mpsc::Receiver<TrayInputEvent>, u32)> {
    //     let mut tray_item = os::tray_item()?;
    let (tray_item_send, tray_item_recv) = mpsc::sync_channel(2);
    //     let tray_item_send_cloned = tray_item_send.clone();
    //     tray_item.add_menu_item("Quit", move || {
    //         tray_item_send_cloned.just_send(TrayInputEvent::Quit)
    //     })?;
    Ok((tray_item_recv, 0))
}

fn main() -> Result<()> {
    let (tray_item_recv, _) = init_tray_item()?;

    let mut break_notifier = break_notifier::BasicTimeBreak::new(
        BreakState::NotBreak,
        Duration::from_secs(5),
        Duration::from_secs(5),
    );

    let (break_send, break_recv) = mpsc::sync_channel(2);
    let mut is_break = false;

    let break_send_c = break_send.clone();
    break_notifier.set_start_break_callback(Some(move || break_send_c.just_send(true)));
    break_notifier.set_end_break_callback(Some(move || break_send.just_send(false)));

    main_loop_run(|world| {
        break_notifier.update(world);

        if let Some(tray_input_event) = tray_item_recv.maybe_recv().break_res_err()? {
            match tray_input_event {
                TrayInputEvent::Quit => return ControlFlow::Break(Ok(())),
                TrayInputEvent::RestartWork => {
                    break_notifier.skip_to(BreakState::Break, Duration::ZERO);
                    is_break = false;
                }
            }
        }
        if let Some(recv_is_break) = break_recv.maybe_recv().break_res_err()? {
            is_break = recv_is_break;
        }
        let _res = os::block_input(is_break);
        ControlFlow::Continue(())
    })?;

    Ok(())
}

trait ResultIntoControLFlow<T, E> {
    fn break_err(self) -> ControlFlow<E, T>;
    fn break_res_err<TB>(self) -> ControlFlow<Result<TB, E>, T>;
}

impl<T, E> ResultIntoControLFlow<T, E> for Result<T, E> {
    fn break_err(self) -> ControlFlow<E, T> {
        match self {
            Ok(o) => ControlFlow::Continue(o),
            Err(e) => ControlFlow::Break(e),
        }
    }

    fn break_res_err<TB>(self) -> ControlFlow<Result<TB, E>, T> {
        match self {
            Ok(o) => ControlFlow::Continue(o),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }
}

trait MpscRecvExt<T> {
    fn maybe_recv(&self) -> std::result::Result<Option<T>, mpsc::TryRecvError>;
}

impl<T> MpscRecvExt<T> for mpsc::Receiver<T> {
    fn maybe_recv(&self) -> std::result::Result<Option<T>, mpsc::TryRecvError> {
        match self.try_recv() {
            Ok(o) => Ok(Some(o)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

trait MpscSendExt<T> {
    fn just_send(&self, v: T);
}

impl<T> MpscSendExt<T> for mpsc::SyncSender<T> {
    fn just_send(&self, v: T) {
        let _res = self.send(v);
    }
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
