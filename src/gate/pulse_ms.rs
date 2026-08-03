use crate::config_storage;

pub(crate) fn pulse_ms() -> u64 {
    config_storage::gate_pulse_ms()
}