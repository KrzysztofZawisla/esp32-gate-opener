pub fn valid_telemetry_interval_s(value: u64) -> bool {
    (1..=3600).contains(&value)
}
