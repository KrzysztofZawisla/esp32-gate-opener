use core::sync::atomic::Ordering;

use super::FAULT;

pub fn clear_fault(mask: u8) {
    FAULT.fetch_and(!mask, Ordering::Relaxed);
}