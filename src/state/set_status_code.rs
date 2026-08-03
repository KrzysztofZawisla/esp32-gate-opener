use core::sync::atomic::Ordering;

use super::STATUS_CODE;
use crate::pure::Status;

pub fn set_status_code(code: Status) {
    STATUS_CODE.store(code.bits(), Ordering::Relaxed);
}
