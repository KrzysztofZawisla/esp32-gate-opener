use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;

use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition, EspNvs};
use log::{info, warn};

use crate::config;

const NVS_NAMESPACE: &str = "gate";
const KEY_BATTERY_MIN_PCT: &str = "batt_min_pct";
const KEY_GRACE_MS: &str = "grace_ms";
const KEY_MOTION_TIMEOUT_S: &str = "motion_timeout";
const KEY_GATE_PULSE_MS: &str = "pulse_ms";
const KEY_TELEMETRY_INTERVAL_S: &str = "tele_interval";
const KEY_HTTP_API_KEY: &str = "api_key";

static NVS: Mutex<Option<EspDefaultNvs>> = Mutex::new(None);

static BATTERY_MIN_PCT: AtomicU8 = AtomicU8::new(0);
static GRACE_MS: AtomicU16 = AtomicU16::new(0);
static MOTION_TIMEOUT_S: AtomicU16 = AtomicU16::new(0);
static GATE_PULSE_MS: AtomicU32 = AtomicU32::new(0);
static TELEMETRY_INTERVAL_S: AtomicU32 = AtomicU32::new(0);
static HTTP_API_KEY: Mutex<String> = Mutex::new(String::new());

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

fn load_all() {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return;
    };

    if let Ok(Some(v)) = nvs.get_u8(KEY_BATTERY_MIN_PCT) {
        BATTERY_MIN_PCT.store(v, Ordering::Relaxed);
    }
    if let Ok(Some(v)) = nvs.get_u16(KEY_GRACE_MS) {
        GRACE_MS.store(v, Ordering::Relaxed);
    }
    if let Ok(Some(v)) = nvs.get_u16(KEY_MOTION_TIMEOUT_S) {
        MOTION_TIMEOUT_S.store(v, Ordering::Relaxed);
    }
    if let Ok(Some(v)) = nvs.get_u32(KEY_GATE_PULSE_MS) {
        GATE_PULSE_MS.store(v, Ordering::Relaxed);
    }
    if let Ok(Some(v)) = nvs.get_u32(KEY_TELEMETRY_INTERVAL_S) {
        TELEMETRY_INTERVAL_S.store(v, Ordering::Relaxed);
    }

    let mut buf = [0u8; 64];
    if let Ok(Some(v)) = nvs.get_str(KEY_HTTP_API_KEY, &mut buf) {
        *HTTP_API_KEY.lock().unwrap() = v.to_string();
    }
}

pub fn battery_min_pct() -> u8 {
    BATTERY_MIN_PCT.load(Ordering::Relaxed)
}

pub fn grace_ms() -> u16 {
    GRACE_MS.load(Ordering::Relaxed)
}

pub fn motion_timeout_s() -> u16 {
    MOTION_TIMEOUT_S.load(Ordering::Relaxed)
}

pub fn gate_pulse_ms() -> u64 {
    GATE_PULSE_MS.load(Ordering::Relaxed) as u64
}

pub fn telemetry_interval_s() -> u64 {
    TELEMETRY_INTERVAL_S.load(Ordering::Relaxed) as u64
}

pub fn http_api_key() -> String {
    HTTP_API_KEY.lock().unwrap().clone()
}

pub fn set_battery_min_pct(value: u8) -> bool {
    persist_u8(KEY_BATTERY_MIN_PCT, value, &BATTERY_MIN_PCT)
}

pub fn set_grace_ms(value: u16) -> bool {
    persist_u16(KEY_GRACE_MS, value, &GRACE_MS)
}

pub fn set_motion_timeout_s(value: u16) -> bool {
    persist_u16(KEY_MOTION_TIMEOUT_S, value, &MOTION_TIMEOUT_S)
}

pub fn set_gate_pulse_ms(value: u64) -> bool {
    persist_u32(KEY_GATE_PULSE_MS, value as u32, &GATE_PULSE_MS)
}

pub fn set_telemetry_interval_s(value: u64) -> bool {
    persist_u32(KEY_TELEMETRY_INTERVAL_S, value as u32, &TELEMETRY_INTERVAL_S)
}

pub fn set_http_api_key(value: &str) -> bool {
    let mut guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_mut() else {
        return false;
    };
    match nvs.set_str(KEY_HTTP_API_KEY, value) {
        Ok(()) => {
            *HTTP_API_KEY.lock().unwrap() = value.to_string();
            true
        }
        Err(e) => {
            warn!("Failed to persist api_key: {e}");
            false
        }
    }
}

fn persist_u8(key: &str, value: u8, slot: &AtomicU8) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u8(key, value) {
        Ok(()) => {
            slot.store(value, Ordering::Relaxed);
            true
        }
        Err(e) => {
            warn!("Failed to persist {key}: {e}");
            false
        }
    }
}

fn persist_u16(key: &str, value: u16, slot: &AtomicU16) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u16(key, value) {
        Ok(()) => {
            slot.store(value, Ordering::Relaxed);
            true
        }
        Err(e) => {
            warn!("Failed to persist {key}: {e}");
            false
        }
    }
}

fn persist_u32(key: &str, value: u32, slot: &AtomicU32) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u32(key, value) {
        Ok(()) => {
            slot.store(value, Ordering::Relaxed);
            true
        }
        Err(e) => {
            warn!("Failed to persist {key}: {e}");
            false
        }
    }
}

fn default_battery_min_pct() -> u8 {
    option_env!("BATTERY_MIN_PCT")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
}

fn default_grace_ms() -> u16 {
    option_env!("GRACE_MS")
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

fn default_motion_timeout_s() -> u16 {
    option_env!("MOTION_TIMEOUT_S")
        .and_then(|v| v.parse().ok())
        .unwrap_or(45)
}

fn default_telemetry_interval_s() -> u64 {
    option_env!("TELEMETRY_INTERVAL_S")
        .and_then(|v| v.parse().ok())
        .unwrap_or(60)
}

fn default_http_api_key() -> &'static str {
    option_env!("HTTP_API_KEY").unwrap_or("")
}
