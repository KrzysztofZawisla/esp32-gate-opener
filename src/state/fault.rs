use core::sync::atomic::Ordering;

use crate::pure::Fault;

use super::FAULT;

pub fn fault() -> Fault {
    Fault::from_bits_retain(FAULT.load(Ordering::Relaxed))
}
