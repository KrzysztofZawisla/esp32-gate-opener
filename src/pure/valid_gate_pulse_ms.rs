pub fn valid_gate_pulse_ms(value: u64) -> bool {
    (1..=60_000).contains(&value)
}