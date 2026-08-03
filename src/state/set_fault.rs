use core::sync::atomic::Ordering;

use super::FAULT;

pub fn set_fault(mask: u8) {
    FAULT.store(mask, Ordering::Relaxed);
}