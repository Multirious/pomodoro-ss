#![allow(unused)]
// #![windows_subsystem = "windows"]

// use std::{
//     ops::ControlFlow,
//     sync::mpsc,
//     time::{Duration, Instant, SystemTime},
// };

// use anyhow::{bail, Error, Result};
// use device_query::{DeviceQuery, DeviceState};

// use activity_monitor::{ActivityKind, ActivityMonitor};
// use break_notifier::BreakState;
// use time::{Stopwatch, Timer};
// use tray_icon::{TrayInputEvent, TrayItem, TrayItemMode};
// use utils::*;

mod activity_monitor;
mod break_notifier;
mod notification;
mod schedule;
mod time;
mod tray_icon;
mod utils;

mod app;

fn main() {}
