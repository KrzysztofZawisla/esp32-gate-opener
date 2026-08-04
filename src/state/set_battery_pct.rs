use core::sync::atomic::Ordering;

use super::BATTERY_PCT;

pub fn set_battery_pct(percentage: u8) {
    BATTERY_PCT.store(percentage, Ordering::Relaxed);
}
