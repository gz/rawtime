mod rtc;
pub mod tsc;

use crate::DateTime;

pub fn wallclock() -> DateTime {
    unsafe { rtc::now() }
}

pub fn precise_time_ns() -> u64 {
    tsc::precise_time_ns()
}
