pub(crate) fn default_battery_min_pct() -> u8 {
    option_env!("BATTERY_MIN_PCT")
        .and_then(|value| value.parse().ok())
        .unwrap_or(20)
}
