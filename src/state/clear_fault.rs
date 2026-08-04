use core::sync::atomic::Ordering;

use crate::pure::Fault;

use super::FAULT;

pub fn clear_fault(mask: Fault) {
    FAULT.fetch_and(!mask.bits(), Ordering::Relaxed);
}
