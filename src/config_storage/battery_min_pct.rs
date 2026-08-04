use core::sync::atomic::Ordering;

use super::BATTERY_MIN_PCT;

pub fn battery_min_pct() -> u8 {
    BATTERY_MIN_PCT.load(Ordering::Relaxed)
}
