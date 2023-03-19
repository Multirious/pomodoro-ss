// #![windows_subsystem = "windows"]

use std::any::{Any, TypeId};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::{mpsc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use device_query::{DeviceQuery, DeviceState};
use once_cell::sync::{Lazy, OnceCell};
use time::{Stopwatch, Timer};
use typemap_rev::TypeMap;

mod time {
    use std::time::Duration;

    use crate::{Process, World};

    pub struct Stopwatch {
        time: Duration,
        pub pause: bool,
    }

    impl Stopwatch {
        pub fn new(time_start: Duration) -> Stopwatch {
            Stopwatch {
                time: time_start,
                pause: false,
            }
        }

        pub fn time(&self) -> Duration {
            self.time
        }
    }

    impl Process for Stopwatch {
        fn update(&mut self, world: &mut World) {
            if self.pause {
                return;
            }
            self.time = self.time.saturating_add(world.delta());
        }
    }

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
    }

    impl Process for Timer {
        fn update(&mut self, world: &mut World) {
            if self.pause {
                return;
            }
            self.time_left = self.time_left.saturating_sub(world.delta());
        }
    }
}

trait Process {
    fn update(&mut self, world: &mut World);
}

struct World {
    delta: Duration,
    events: Vec<()>,
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
    WorktTime20,
    WorktTime10,
}

struct ActivityMonitor {
    data: Vec<(ActivityKind, Instant, f64)>,
    total_activity_value: f64,
    time_start: Instant,
    previous_mouse_coord: Vec2,
    calulate_activity_fn: CalculateActivityFn,
}

#[derive(Clone, Copy)]
enum ActivityKind {
    KeyPress,
    MouseClick,
    MouseMove { pixels: Vec2 },
}

type CalculateActivityFn = Box<dyn Fn(&ActivityMonitor, ActivityKind) -> f64>;

impl ActivityMonitor {
    pub fn new(world: &World, calulate_activity_fn: CalculateActivityFn) -> ActivityMonitor {
        ActivityMonitor {
            data: Vec::new(),
            total_activity_value: 0.0,
            time_start: Instant::now(),
            previous_mouse_coord: DeviceState.get_mouse().coords.into(),
            calulate_activity_fn,
        }
    }

    pub fn previous_mouse_coord(&self) -> Vec2 {
        self.previous_mouse_coord
    }

    pub fn activity_value(&self) -> f64 {
        self.total_activity_value
    }

    pub fn data(&self) -> &Vec<(ActivityKind, Instant, f64)> {
        &self.data
    }

    pub fn time_start(&self) -> Instant {
        self.time_start
    }

    pub fn averge_activity_value(&self, duration: Duration) -> Option<f64> {
        let get_data_after = Instant::now() - duration;
        if self.data.is_empty() {
            return None;
        }
        let index_data_just_before_requested =
            self.data
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, (_, when, _))| {
                    if when < &get_data_after {
                        Some(i)
                    } else {
                        None
                    }
                })?;
        if index_data_just_before_requested == self.data.len() - 1 {
            return None;
        }
        let index = index_data_just_before_requested + 1;
        let averge: f64 = self.data[index..]
            .iter()
            .map(|(_, _, val)| val)
            .sum::<f64>()
            / self.data.len() as f64;
        Some(averge)
    }

    pub fn averge_all_activity_value(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        Some(self.total_activity_value / self.data.len() as f64)
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.total_activity_value = 0.0;
    }

    pub fn update_activity(&mut self, activity: ActivityKind, coeff: f64) {
        let val = (self.calulate_activity_fn)(self, activity);
        self.data.push((activity, Instant::now(), val));
        self.total_activity_value += val * coeff;
    }
}

impl Process for ActivityMonitor {
    fn update(&mut self, world: &mut World) {
        let mouse = DeviceState.get_mouse();
        let mouse_coord = mouse.coords.into();
        let keys = DeviceState.get_keys();
        let mouse_diff = self.previous_mouse_coord - mouse_coord;
        self.update_activity(ActivityKind::MouseClick, mouse.button_pressed.len() as f64);
        if mouse_diff.magnitude() > 0.0 {
            self.update_activity(ActivityKind::MouseMove { pixels: mouse_diff }, 1.0)
        }
        self.update_activity(ActivityKind::KeyPress, keys.len() as f64);

        self.previous_mouse_coord = mouse_coord;
    }
}

#[derive(Default, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AllSafePhantomData<T: ?Sized>(pub PhantomData<T>);

impl<T: ?Sized> AllSafePhantomData<T> {
    pub fn new() -> AllSafePhantomData<T> {
        AllSafePhantomData(PhantomData)
    }
}

impl<T: ?Sized> Clone for AllSafePhantomData<T> {
    fn clone(&self) -> Self {
        AllSafePhantomData(PhantomData)
    }
}

unsafe impl<T: ?Sized> Send for AllSafePhantomData<T> {}
unsafe impl<T: ?Sized> Sync for AllSafePhantomData<T> {}

// trait DynKey {
//     fn eq(&self, other: &dyn DynKey) -> bool;
//     fn hash(&self) -> u64;
//     fn as_any(&self) -> &dyn Any;
// }

// impl<T: Eq + Hash + 'static> DynKey for T {
//     fn eq(&self, other: &dyn DynKey) -> bool {
//         if let Some(other) = other.as_any().downcast_ref::<T>() {
//             return self == other;
//         }
//         false
//     }

//     fn hash(&self) -> u64 {
//         let mut h = DefaultHasher::new();
//         // mix the typeid of T into the hash to make distinct types
//         // provide distinct hashes
//         Hash::hash(&(TypeId::of::<T>(), self), &mut h);
//         h.finish()
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }
// }

trait HierarchyTrait {
    const HIERARCHY: Hierarchy = {
        Hierarchy {
            parents: Self::parents,
            ancestors: Self::ancestors,
        }
    };

    fn parents_static() -> &'static ParentsStatic;
    fn parents() -> Parents {
        let ParentsStatic(parents_static, init) = Self::parents_static();
        Parents(parents_static.get_or_init(|| init().0))
    }

    fn ancestors_static() -> &'static AncestorsStatic;
    fn ancestors() -> Ancestors {
        Ancestors(Self::ancestors_static().0.get_or_init(|| {
            Self::parents()
                .0
                .iter()
                .flat_map(|(id, grandparent)| {
                    std::iter::once(*id).chain((grandparent.ancestors)().0.iter().copied())
                })
                .collect()
        }))
    }
}

trait ObjSafeHierarchyTrait {
    // fn hierarchy(&self) -> Hierarchy {
    //     Hierarchy {
    //         parents: || self.parents,
    //         ancestors: || self.ancestors,
    //     }
    // }
    fn parents_static(&self) -> &'static ParentsStatic;
    fn parents(&self) -> Parents {
        let ParentsStatic(parents_static, init) = self.parents_static();
        Parents(parents_static.get_or_init(|| init().0))
    }

    fn ancestors_static(&self) -> &'static AncestorsStatic;
    fn ancestors(&self) -> Ancestors {
        Ancestors(self.ancestors_static().0.get_or_init(|| {
            self.parents()
                .0
                .iter()
                .flat_map(|(id, grandparent)| {
                    std::iter::once(*id).chain((grandparent.ancestors)().0.iter().copied())
                })
                .collect()
        }))
    }
}

struct DynTest(&'static dyn ObjSafeHierarchyTrait);
#[derive(Debug, Clone, Copy)]
struct Parents(&'static HashMap<TypeId, &'static Hierarchy>);
#[derive(Debug)]
struct ParentsStatic(
    OnceCell<HashMap<TypeId, &'static Hierarchy>>,
    fn() -> ParentsStaticBuilder,
);
#[derive(Debug)]
struct ParentsStaticBuilder(HashMap<TypeId, &'static Hierarchy>);
struct ObjSafeParentsStaticBuilder(HashMap<TypeId, &'static Hierarchy>);
#[derive(Debug, Clone, Copy)]
struct Ancestors(&'static HashSet<TypeId>);
#[derive(Debug)]
struct AncestorsStatic(OnceCell<HashSet<TypeId>>);

impl AncestorsStatic {
    pub const fn new() -> AncestorsStatic {
        AncestorsStatic(OnceCell::new())
    }
}

impl ParentsStatic {
    pub const fn new(init: fn() -> ParentsStaticBuilder) -> ParentsStatic {
        ParentsStatic(OnceCell::new(), init)
    }
}
impl ParentsStaticBuilder {
    pub fn new() -> ParentsStaticBuilder {
        ParentsStaticBuilder(HashMap::new())
    }

    pub fn add_parent<P: HierarchyTrait + 'static>(mut self) -> Self {
        self.0.insert(TypeId::of::<P>(), &P::HIERARCHY);
        self
    }
}
impl ObjSafeParentsStaticBuilder {
    pub fn new() -> ParentsStaticBuilder {
        ParentsStaticBuilder(HashMap::new())
    }

    pub fn add_parent(mut self, parent: &dyn ObjSafeHierarchyTrait) -> Self {
        self.0.insert(
            TypeId::of::<P>(),
            &Hierarchy {
                parents: || parent.parents,
                ancestors: || parent.ancestors,
            },
        );
        self
    }
}

#[derive(Debug, Clone, Copy)]
struct Hierarchy {
    parents: fn() -> Parents,
    ancestors: fn() -> Ancestors,
}

#[test]
fn test_event() {
    #[derive(Debug)]
    struct AllEvent;
    #[derive(Debug)]
    struct SomeEvent;
    #[derive(Debug)]
    struct DeeperEvent;

    impl ObjSafeHierarchyTrait for AllEvent {
        fn parents_static(&self) -> &'static ParentsStatic {
            static P: ParentsStatic = ParentsStatic::new(ParentsStaticBuilder::new);
            &P
        }

        fn ancestors_static(&self) -> &'static AncestorsStatic {
            static A: AncestorsStatic = AncestorsStatic::new();
            &A
        }
    }

    impl ObjSafeHierarchyTrait for SomeEvent {
        fn parents_static(&self) -> &'static ParentsStatic {
            static P: ParentsStatic =
                ParentsStatic::new(|| ParentsStaticBuilder::new().add_parent::<AllEvent>());
            &P
        }

        fn ancestors_static(&self) -> &'static AncestorsStatic {
            static A: AncestorsStatic = AncestorsStatic::new();
            &A
        }
    }

    impl HierarchyTrait for DeeperEvent {
        fn parents_static() -> &'static ParentsStatic {
            static P: ParentsStatic =
                ParentsStatic::new(|| ParentsStaticBuilder::new().add_parent::<SomeEvent>());
            &P
        }

        fn ancestors_static() -> &'static AncestorsStatic {
            static A: AncestorsStatic = AncestorsStatic::new();
            &A
        }
    }

    println!("before calling main fn");
    dbg!(SomeEvent::ancestors());
    dbg!(DeeperEvent::ancestors());
    println!("after calling main fn");
}

fn main() -> Result<()> {
    let (tray_item_send, tray_item_recv) = mpsc::sync_channel(10);
    let mut tray_item = tray_item()?;
    let tray_item_send_cloned = tray_item_send.clone();
    tray_item.add_menu_item("Quit", move || {
        let _result = tray_item_send_cloned.try_send(TrayInputEvent::Quit);
    })?;
    tray_item.add_menu_item("Restart work", move || {
        let _result = tray_item_send.try_send(TrayInputEvent::RestartWork);
    })?;

    let mut world = World {
        delta: Duration::ZERO,
        events: vec![],
    };

    let mut activity_monitor = ActivityMonitor::new(
        &world,
        Box::new(|_, activity_kind| match activity_kind {
            ActivityKind::KeyPress => 1.0,
            ActivityKind::MouseClick => 1.0,
            ActivityKind::MouseMove { pixels } => pixels.magnitude(),
        }),
    );

    let time_break = Duration::from_secs(60 * 10);
    let time_work = Duration::from_secs(60 * 20);
    let time_warn_before_break = Duration::from_secs(60 * 5);
    let time_alert_before_break = Duration::from_secs(30);
    let time_loop = time_work + time_break;

    let mut pomodoro_state = PomodoroState::Work;

    let mut current_working_time: Timer = Timer::new(time_work);
    let mut current_break_time: Stopwatch = Stopwatch::new(Duration::ZERO);

    let mut last_frame_time = Instant::now();

    loop {
        world.delta = last_frame_time.elapsed();

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

        last_frame_time = Instant::now();
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Vec2 {
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
