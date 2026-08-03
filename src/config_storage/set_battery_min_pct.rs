use super::{persist_u8, BATTERY_MIN_PCT, KEY_BATTERY_MIN_PCT};

pub fn set_battery_min_pct(value: u8) -> bool {
    persist_u8(KEY_BATTERY_MIN_PCT, value, &BATTERY_MIN_PCT)
}