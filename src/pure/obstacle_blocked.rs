pub fn obstacle_blocked(pin_is_high: bool, active_high: bool) -> bool {
    pin_is_high == active_high
}
