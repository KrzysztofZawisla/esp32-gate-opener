pub fn battery_divider_ratio() -> f32 {
    option_env!("BATTERY_DIVIDER_RATIO")
        .and_then(|value| value.parse().ok())
        .unwrap_or(6.0)
}
