use core::sync::atomic::Ordering;

use super::STATUS_CODE;
use crate::pure::Status;

pub fn status() -> &'static str {
    Status::from_raw(STATUS_CODE.load(Ordering::Relaxed)).as_str()
}
