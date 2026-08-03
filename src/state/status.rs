use core::sync::atomic::Ordering;

use super::STATUS_CODE;
use crate::pure::status_str;

pub fn status() -> &'static str {
    status_str(STATUS_CODE.load(Ordering::Relaxed))
}
