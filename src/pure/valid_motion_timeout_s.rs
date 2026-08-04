pub fn valid_motion_timeout_s(value: u16) -> bool {
    (1..=300).contains(&value)
}