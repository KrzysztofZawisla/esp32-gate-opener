use core::sync::atomic::Ordering;

use super::MQTT_CONNECTED;

pub fn set_mqtt_connected(connected: bool) {
    MQTT_CONNECTED.store(connected, Ordering::Relaxed);
}
