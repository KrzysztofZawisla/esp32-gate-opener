use core::sync::atomic::Ordering;

use super::STATUS_CODE;

pub fn set_status_code(code: u8) {
    STATUS_CODE.store(code, Ordering::Relaxed);
}
