use core::sync::atomic::Ordering;

use super::GRACE_MS;

pub fn grace_ms() -> u16 {
    GRACE_MS.load(Ordering::Relaxed)
}