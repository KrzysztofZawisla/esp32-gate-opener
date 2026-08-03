use super::{persist_u16, KEY_MOTION_TIMEOUT_S, MOTION_TIMEOUT_S};

pub fn set_motion_timeout_s(value: u16) -> bool {
    persist_u16(KEY_MOTION_TIMEOUT_S, value, &MOTION_TIMEOUT_S)
}
