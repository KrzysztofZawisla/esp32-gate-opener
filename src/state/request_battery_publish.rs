use core::sync::atomic::Ordering;

use super::BATTERY_PUBLISH_REQUEST;

pub fn request_battery_publish() {
    BATTERY_PUBLISH_REQUEST.store(true, Ordering::Relaxed);
}