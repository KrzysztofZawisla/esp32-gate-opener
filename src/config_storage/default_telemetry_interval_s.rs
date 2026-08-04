pub(crate) fn default_telemetry_interval_s() -> u64 {
    option_env!("TELEMETRY_INTERVAL_S")
        .and_then(|value| value.parse().ok())
        .unwrap_or(60)
}
