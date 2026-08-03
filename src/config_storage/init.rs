use core::sync::atomic::Ordering;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use log::{info, warn};

use crate::config;

use super::{
    default_battery_min_pct, default_grace_ms, default_http_api_key, default_motion_timeout_s,
    default_telemetry_interval_s, load_all, BATTERY_MIN_PCT, GATE_PULSE_MS, GRACE_MS, HTTP_API_KEY,
    MOTION_TIMEOUT_S, NVS, NVS_NAMESPACE, TELEMETRY_INTERVAL_S,
};

pub fn init(partition: EspDefaultNvsPartition) {
    BATTERY_MIN_PCT.store(default_battery_min_pct(), Ordering::Relaxed);
    GRACE_MS.store(default_grace_ms(), Ordering::Relaxed);
    MOTION_TIMEOUT_S.store(default_motion_timeout_s(), Ordering::Relaxed);
    GATE_PULSE_MS.store(config::GATE_PULSE_MS as u32, Ordering::Relaxed);
    TELEMETRY_INTERVAL_S.store(default_telemetry_interval_s() as u32, Ordering::Relaxed);
    *HTTP_API_KEY.lock().unwrap() = default_http_api_key().to_string();

    match EspNvs::new(partition, NVS_NAMESPACE, true) {
        Ok(nvs) => {
            info!("Runtime config opened in NVS");
            *NVS.lock().unwrap() = Some(nvs);
            load_all();
        }
        Err(e) => warn!("Failed to open NVS runtime config: {e}; using defaults"),
    }
}