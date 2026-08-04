use core::sync::atomic::Ordering;

use super::{
    BATTERY_MIN_PCT, GATE_PULSE_MS, GRACE_MS, HTTP_API_KEY, KEY_BATTERY_MIN_PCT, KEY_GATE_PULSE_MS,
    KEY_GRACE_MS, KEY_HTTP_API_KEY, KEY_MOTION_TIMEOUT_S, KEY_TELEMETRY_INTERVAL_S,
    MOTION_TIMEOUT_S, NVS, TELEMETRY_INTERVAL_S,
};

pub fn load_all() {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return;
    };

    if let Ok(Some(value)) = nvs.get_u8(KEY_BATTERY_MIN_PCT) {
        BATTERY_MIN_PCT.store(value, Ordering::Relaxed);
    }
    if let Ok(Some(value)) = nvs.get_u16(KEY_GRACE_MS) {
        GRACE_MS.store(value, Ordering::Relaxed);
    }
    if let Ok(Some(value)) = nvs.get_u16(KEY_MOTION_TIMEOUT_S) {
        MOTION_TIMEOUT_S.store(value, Ordering::Relaxed);
    }
    if let Ok(Some(value)) = nvs.get_u32(KEY_GATE_PULSE_MS) {
        GATE_PULSE_MS.store(value, Ordering::Relaxed);
    }
    if let Ok(Some(value)) = nvs.get_u32(KEY_TELEMETRY_INTERVAL_S) {
        TELEMETRY_INTERVAL_S.store(value, Ordering::Relaxed);
    }

    let mut buffer = [0u8; 64];
    if let Ok(Some(value)) = nvs.get_str(KEY_HTTP_API_KEY, &mut buffer) {
        *HTTP_API_KEY.lock().unwrap() = value.to_string();
    }
}
