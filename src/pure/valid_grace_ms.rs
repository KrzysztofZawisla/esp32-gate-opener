pub fn valid_grace_ms(value: u16) -> bool {
    value <= 60_000
}
