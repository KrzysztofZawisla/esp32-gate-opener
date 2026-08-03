pub fn valid_battery_min_pct(value: u8) -> bool {
    value <= 100
}

pub fn valid_grace_ms(value: u16) -> bool {
    value <= 60_000
}

pub fn valid_motion_timeout_s(value: u16) -> bool {
    (1..=300).contains(&value)
}

pub fn valid_gate_pulse_ms(value: u64) -> bool {
    (1..=60_000).contains(&value)
}

pub fn valid_telemetry_interval_s(value: u64) -> bool {
    (1..=3600).contains(&value)
}
