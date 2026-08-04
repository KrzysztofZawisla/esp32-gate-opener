pub(crate) fn default_grace_ms() -> u16 {
    option_env!("GRACE_MS")
        .and_then(|value| value.parse().ok())
        .unwrap_or(300)
}
