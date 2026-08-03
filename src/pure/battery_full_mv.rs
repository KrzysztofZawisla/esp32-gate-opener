pub fn battery_full_mv() -> f32 {
    option_env!("BATTERY_FULL_MV")
        .and_then(|value| value.parse().ok())
        .unwrap_or(12600.0)
}
