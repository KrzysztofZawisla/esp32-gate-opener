use core::sync::atomic::Ordering;

use super::BATTERY_PUBLISH_REQUEST;

pub fn take_battery_publish_request() -> bool {
    BATTERY_PUBLISH_REQUEST.swap(false, Ordering::Relaxed)
}