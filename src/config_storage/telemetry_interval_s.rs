use core::sync::atomic::Ordering;

use super::TELEMETRY_INTERVAL_S;

pub fn telemetry_interval_s() -> u64 {
    TELEMETRY_INTERVAL_S.load(Ordering::Relaxed) as u64
}
