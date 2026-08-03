use super::{persist_u32, GATE_PULSE_MS, KEY_GATE_PULSE_MS};

pub fn set_gate_pulse_ms(value: u64) -> bool {
    persist_u32(KEY_GATE_PULSE_MS, value as u32, &GATE_PULSE_MS)
}