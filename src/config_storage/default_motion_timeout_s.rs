pub(crate) fn default_motion_timeout_s() -> u16 {
    option_env!("MOTION_TIMEOUT_S")
        .and_then(|v| v.parse().ok())
        .unwrap_or(45)
}