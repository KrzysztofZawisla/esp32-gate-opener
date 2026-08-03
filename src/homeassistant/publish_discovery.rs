use crate::config::{
    AVAILABILITY_TOPIC, BATTERY_TOPIC, BATTERY_VOLTAGE_TOPIC, COMMAND_TOPIC, FAULT_TOPIC,
    OBSTACLE_TOPIC, STATUS_TOPIC,
};
use crate::pure::{discovery_configs, DiscoveryTopics};

use super::publish_raw;

pub(crate) fn publish_discovery() {
    let topics = DiscoveryTopics {
        command: COMMAND_TOPIC,
        status: STATUS_TOPIC,
        availability: AVAILABILITY_TOPIC,
        battery: BATTERY_TOPIC,
        battery_voltage: BATTERY_VOLTAGE_TOPIC,
        obstacle: OBSTACLE_TOPIC,
        fault: FAULT_TOPIC,
    };
    for config in discovery_configs("esp32_gate_opener", &topics) {
        publish_raw(&config.topic, config.payload.as_bytes(), true);
    }
}
