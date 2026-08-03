use core::sync::atomic::Ordering;

use log::info;

use crate::config;

use super::{
    default_battery_min_pct, default_grace_ms, default_http_api_key, default_motion_timeout_s,
    default_telemetry_interval_s, BATTERY_MIN_PCT, GATE_PULSE_MS, GRACE_MS, HTTP_API_KEY,
    KEY_BATTERY_MIN_PCT, KEY_GATE_PULSE_MS, KEY_GRACE_MS, KEY_HTTP_API_KEY, KEY_MOTION_TIMEOUT_S,
    KEY_TELEMETRY_INTERVAL_S, MOTION_TIMEOUT_S, NVS, TELEMETRY_INTERVAL_S,
};

/// Drops every persisted runtime value and restores the compile-time defaults.
///
/// This is what `POST /config?reset=1` triggers: instead of rewriting NVS with
/// the defaults, the keys are removed so a later firmware upgrade that changes
/// a default picks up the new value instead of an old persisted one.
pub fn reset_all() {
    if let Some(nvs) = NVS.lock().unwrap().as_mut() {
        let _ = nvs.remove(KEY_BATTERY_MIN_PCT);
        let _ = nvs.remove(KEY_GRACE_MS);
        let _ = nvs.remove(KEY_MOTION_TIMEOUT_S);
        let _ = nvs.remove(KEY_GATE_PULSE_MS);
        let _ = nvs.remove(KEY_TELEMETRY_INTERVAL_S);
        let _ = nvs.remove(KEY_HTTP_API_KEY);
    }
    restore_defaults();
    info!("Runtime config reset to defaults");
}

/// Restores the compile-time defaults into the in-memory statics. Used by
/// `init()` (before values are loaded from NVS) and after a `reset` request.
/// Does not touch NVS.
pub(crate) fn restore_defaults() {
    BATTERY_MIN_PCT.store(default_battery_min_pct(), Ordering::Relaxed);
    GRACE_MS.store(default_grace_ms(), Ordering::Relaxed);
    MOTION_TIMEOUT_S.store(default_motion_timeout_s(), Ordering::Relaxed);
    GATE_PULSE_MS.store(config::GATE_PULSE_MS as u32, Ordering::Relaxed);
    TELEMETRY_INTERVAL_S.store(default_telemetry_interval_s() as u32, Ordering::Relaxed);
    *HTTP_API_KEY.lock().unwrap() = default_http_api_key().to_string();
}
