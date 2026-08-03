use core::sync::atomic::Ordering;

use super::BATTERY_PCT;

pub fn set_battery_pct(pct: u8) {
    BATTERY_PCT.store(pct, Ordering::Relaxed);
}