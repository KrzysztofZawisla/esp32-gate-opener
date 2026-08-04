use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU8};
use std::sync::Mutex;

use esp_idf_svc::nvs::EspDefaultNvs;

pub(crate) const NVS_NAMESPACE: &str = "gate";
pub(crate) const KEY_BATTERY_MIN_PCT: &str = "batt_min_pct";
pub(crate) const KEY_GRACE_MS: &str = "grace_ms";
pub(crate) const KEY_MOTION_TIMEOUT_S: &str = "motion_timeout";
pub(crate) const KEY_GATE_PULSE_MS: &str = "pulse_ms";
pub(crate) const KEY_TELEMETRY_INTERVAL_S: &str = "tele_interval";
pub(crate) const KEY_HTTP_API_KEY: &str = "api_key";

pub(crate) static NVS: Mutex<Option<EspDefaultNvs>> = Mutex::new(None);

pub(crate) static BATTERY_MIN_PCT: AtomicU8 = AtomicU8::new(0);
pub(crate) static GRACE_MS: AtomicU16 = AtomicU16::new(0);
pub(crate) static MOTION_TIMEOUT_S: AtomicU16 = AtomicU16::new(0);
pub(crate) static GATE_PULSE_MS: AtomicU32 = AtomicU32::new(0);
pub(crate) static TELEMETRY_INTERVAL_S: AtomicU32 = AtomicU32::new(0);
pub(crate) static HTTP_API_KEY: Mutex<String> = Mutex::new(String::new());

mod battery_min_pct;
mod default_battery_min_pct;
mod default_grace_ms;
mod default_http_api_key;
mod default_motion_timeout_s;
mod default_telemetry_interval_s;
mod gate_pulse_ms;
mod grace_ms;
mod http_api_key;
mod init;
mod load_all;
mod motion_timeout_s;
mod persist_u16;
mod persist_u32;
mod persist_u8;
mod reset_all;
mod set_battery_min_pct;
mod set_gate_pulse_ms;
mod set_grace_ms;
mod set_http_api_key;
mod set_motion_timeout_s;
mod set_telemetry_interval_s;
mod telemetry_interval_s;

pub use battery_min_pct::battery_min_pct;
pub use gate_pulse_ms::gate_pulse_ms;
pub use grace_ms::grace_ms;
pub use http_api_key::http_api_key;
pub use init::init;
pub use motion_timeout_s::motion_timeout_s;
pub use reset_all::reset_all;
pub use set_battery_min_pct::set_battery_min_pct;
pub use set_gate_pulse_ms::set_gate_pulse_ms;
pub use set_grace_ms::set_grace_ms;
pub use set_http_api_key::set_http_api_key;
pub use set_motion_timeout_s::set_motion_timeout_s;
pub use set_telemetry_interval_s::set_telemetry_interval_s;
pub use telemetry_interval_s::telemetry_interval_s;

pub(crate) use load_all::load_all;
pub(crate) use persist_u16::persist_u16;
pub(crate) use persist_u32::persist_u32;
pub(crate) use persist_u8::persist_u8;
pub(crate) use reset_all::restore_defaults;

pub(crate) use default_battery_min_pct::default_battery_min_pct;
pub(crate) use default_grace_ms::default_grace_ms;
pub(crate) use default_http_api_key::default_http_api_key;
pub(crate) use default_motion_timeout_s::default_motion_timeout_s;
pub(crate) use default_telemetry_interval_s::default_telemetry_interval_s;
