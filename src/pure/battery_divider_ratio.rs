pub fn battery_divider_ratio() -> f32 {
    option_env!("BATTERY_DIVIDER_RATIO")
        .and_then(|v| v.parse().ok())
        .unwrap_or(6.0)
}