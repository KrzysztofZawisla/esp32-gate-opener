use core::sync::atomic::Ordering;

use super::BATTERY_PCT;

pub fn battery_pct() -> u8 {
    BATTERY_PCT.load(Ordering::Relaxed)
}