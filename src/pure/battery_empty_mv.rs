pub fn battery_empty_mv() -> f32 {
    option_env!("BATTERY_EMPTY_MV")
        .and_then(|value| value.parse().ok())
        .unwrap_or(11500.0)
}
