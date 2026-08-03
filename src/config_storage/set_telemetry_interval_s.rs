use super::{persist_u32, KEY_TELEMETRY_INTERVAL_S, TELEMETRY_INTERVAL_S};

pub fn set_telemetry_interval_s(value: u64) -> bool {
    persist_u32(KEY_TELEMETRY_INTERVAL_S, value as u32, &TELEMETRY_INTERVAL_S)
}