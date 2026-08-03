use core::sync::atomic::Ordering;

use super::FAULT;

pub fn fault() -> u8 {
    FAULT.load(Ordering::Relaxed)
}