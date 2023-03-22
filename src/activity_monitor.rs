use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use device_query::{DeviceEvents, DeviceQuery, DeviceState, Keycode};

use crate::World;

pub type CalculateActivityFn = Box<dyn Fn(&ActivityMonitor, ActivityKind, usize) -> f64>;

pub struct ActivityMonitor {
    max_data_buffer_size: usize,
    data: VecDeque<(ActivityKind, Instant, f64)>,
    total_activity_value: f64,
    time_start: Instant,
    previous_mouse_coord: (i32, i32),
    previous_key_presses: Vec<Keycode>,
    previous_mouse_pressses: Vec<bool>,
    calulate_activity_fn: CalculateActivityFn,
}

#[derive(Debug, Clone, Copy)]
pub enum ActivityKind {
    KeyPress,
    KeyJustPress,
    MousePressed,
    MouseJustPressed,
    MouseMove { distance: f64 },
}

impl ActivityMonitor {
    pub fn new<A>(calulate_activity_fn: A, max_data_buffer_size: usize) -> ActivityMonitor
    where
        A: Fn(&ActivityMonitor, ActivityKind, usize) -> f64 + 'static,
    {
        ActivityMonitor {
            max_data_buffer_size,
            data: VecDeque::with_capacity(max_data_buffer_size),
            total_activity_value: 0.0,
            time_start: Instant::now(),
            previous_mouse_coord: DeviceState.get_mouse().coords.into(),
            previous_key_presses: DeviceState.get_keys(),
            previous_mouse_pressses: DeviceState.get_mouse().button_pressed,
            calulate_activity_fn: Box::new(calulate_activity_fn),
        }
    }

    pub fn set_calculate_activity_fn<A>(&mut self, calulate_activity_fn: A)
    where
        A: Fn(&ActivityMonitor, ActivityKind, usize) -> f64 + 'static,
    {
        self.calulate_activity_fn = Box::new(calulate_activity_fn)
    }

    pub fn activity_value(&self) -> f64 {
        self.total_activity_value
    }

    pub fn data(&self) -> &VecDeque<(ActivityKind, Instant, f64)> {
        &self.data
    }

    pub fn time_start(&self) -> Instant {
        self.time_start
    }

    pub fn time_last_active(&self) -> Instant {
        self.data[self.data.len()].1
    }

    pub fn activity_rate_in_the_last(&self, duration: Duration) -> (f64, usize) {
        self.activity_value_after(Instant::now() - duration)
    }

    pub fn activity_value_after(&self, after: Instant) -> (f64, usize) {
        if self.data.is_empty() {
            return (0.0, 0);
        }

        let index_data_just_before_requested = self
            .data
            .iter()
            .enumerate()
            .rev()
            .find_map(
                |(i, (_, when, _))| {
                    if when < &after {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
            .unwrap_or(0);
        let data_range = self.data.iter().skip(index_data_just_before_requested + 1);
        let len = data_range.len();
        let rate: f64 = data_range.map(|(_, _, val)| val).sum::<f64>();
        (rate, len)
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.total_activity_value = 0.0;
        self.time_start = Instant::now();
    }

    pub fn update_activity(&mut self, activity: ActivityKind, amount: usize, world: &World) {
        let val = (self.calulate_activity_fn)(self, activity, amount) * world.delta().as_secs_f64();
        if self.data.len() == self.max_data_buffer_size {
            self.data.pop_front();
        }
        self.data.push_back((activity, Instant::now(), val));
        self.total_activity_value += val;
    }

    fn update(&mut self, world: &World) {
        let mouse = DeviceState.get_mouse();
        let mouse_coord = mouse.coords.into();
        let keys = DeviceState.get_keys();
        let mouse_diff = {
            let (px, py) = self.previous_mouse_coord;
            let (nx, ny) = mouse_coord;
            let (dx, dy) = ((px - nx) as f64, (py - ny) as f64);
            (dx * dx + dy * dy).sqrt().abs()
        };
        let mouse_buttons_pressed = mouse.button_pressed.iter().filter(|v| **v).count();
        let mouse_buttons_just_pressed = self
            .previous_mouse_pressses
            .iter()
            .zip(mouse.button_pressed.iter())
            .filter(|(&previous, &new)| !previous && new)
            .count();
        let keys_pressed = keys.len();
        let keys_just_pressed = keys
            .iter()
            .filter(|k| !self.previous_key_presses.contains(k))
            .count();

        if mouse_buttons_pressed > 0 {
            self.update_activity(ActivityKind::MousePressed, mouse_buttons_pressed, world);
        }
        if mouse_buttons_just_pressed > 0 {
            self.update_activity(
                ActivityKind::MouseJustPressed,
                mouse_buttons_just_pressed,
                world,
            );
        }
        if mouse_diff > 0.0 {
            self.update_activity(
                ActivityKind::MouseMove {
                    distance: mouse_diff,
                },
                1,
                world,
            )
        }
        if keys_pressed > 0 {
            self.update_activity(ActivityKind::KeyPress, keys_pressed, world);
        }
        if keys_just_pressed > 0 {
            self.update_activity(ActivityKind::KeyJustPress, keys_just_pressed, world);
        }

        self.previous_mouse_pressses = mouse.button_pressed;
        self.previous_mouse_coord = mouse_coord;
        self.previous_key_presses = keys;
    }
}
