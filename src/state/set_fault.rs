use core::sync::atomic::Ordering;

use crate::pure::Fault;

use super::FAULT;

pub fn set_fault(mask: Fault) {
    FAULT.store(mask.bits(), Ordering::Relaxed);
}
