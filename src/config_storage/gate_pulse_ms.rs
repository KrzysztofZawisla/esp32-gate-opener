use core::sync::atomic::Ordering;

use super::GATE_PULSE_MS;

pub fn gate_pulse_ms() -> u64 {
    GATE_PULSE_MS.load(Ordering::Relaxed) as u64
}
