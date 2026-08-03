use core::sync::atomic::Ordering;

use super::MQTT_CONNECTED;

pub fn mqtt_connected() -> bool {
    MQTT_CONNECTED.load(Ordering::Relaxed)
}