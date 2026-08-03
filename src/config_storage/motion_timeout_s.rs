use core::sync::atomic::Ordering;

use super::MOTION_TIMEOUT_S;

pub fn motion_timeout_s() -> u16 {
    MOTION_TIMEOUT_S.load(Ordering::Relaxed)
}