use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default)]
    struct Day: u8 {
        const SUNDAY    = 0b0000_0001;
        const MONDAY    = 0b0000_0010;
        const TUESDAY   = 0b0000_0100;
        const WEDNESDAY = 0b0000_1000;
        const THURSDAY  = 0b0001_0000;
        const FRIDAY    = 0b0010_0000;
        const SATURDAY  = 0b0100_0000;
    }
}

#[derive(Debug, Default)]
struct Schedule {
    auto_start_at_time: Option<()>,
    auto_start_at_day: Option<Day>,
}

impl Schedule {
    pub fn new() -> Schedule {
        Schedule::default()
    }
}
