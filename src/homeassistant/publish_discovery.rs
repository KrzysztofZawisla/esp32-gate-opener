use serde::Serialize;

use crate::config::{
    AVAILABILITY_TOPIC, BATTERY_TOPIC, BATTERY_VOLTAGE_TOPIC, COMMAND_TOPIC, FAULT_TOPIC,
    OBSTACLE_TOPIC, STATUS_TOPIC,
};

use super::publish_raw;

#[derive(Clone, Serialize)]
struct Device {
    #[serde(rename = "identifiers")]
    id: [&'static str; 1],
    name: &'static str,
    manufacturer: &'static str,
    model: &'static str,
}

#[derive(Serialize)]
struct Cover {
    name: &'static str,
    unique_id: String,
    device: Device,
    command_topic: &'static str,
    state_topic: &'static str,
    availability_topic: &'static str,
    payload_open: &'static str,
    payload_close: &'static str,
    device_class: &'static str,
}

#[derive(Serialize)]
struct Sensor {
    name: &'static str,
    unique_id: String,
    device: Device,
    state_topic: &'static str,
    unit_of_measurement: &'static str,
    device_class: &'static str,
}

#[derive(Serialize)]
struct BinarySensor {
    name: &'static str,
    unique_id: String,
    device: Device,
    state_topic: &'static str,
    device_class: &'static str,
    payload_on: &'static str,
    payload_off: &'static str,
}

pub(crate) fn publish_discovery() {
    let unique_id = "esp32_gate_opener";
    let device = Device {
        id: [unique_id],
        name: "Gate",
        manufacturer: "ESP32",
        model: "Gate Opener",
    };

    let cover = Cover {
        name: "Gate",
        unique_id: format!("{unique_id}_cover"),
        device: device.clone(),
        command_topic: COMMAND_TOPIC,
        state_topic: STATUS_TOPIC,
        availability_topic: AVAILABILITY_TOPIC,
        payload_open: "open",
        payload_close: "close",
        device_class: "gate",
    };
    publish_raw(
        &format!("homeassistant/cover/{unique_id}/config"),
        serde_json::to_string(&cover).unwrap_or_default().as_bytes(),
        true,
    );

    let battery = Sensor {
        name: "Gate battery",
        unique_id: format!("{unique_id}_battery"),
        device: device.clone(),
        state_topic: BATTERY_TOPIC,
        unit_of_measurement: "%",
        device_class: "battery",
    };
    publish_raw(
        &format!("homeassistant/sensor/{unique_id}/battery/config"),
        serde_json::to_string(&battery)
            .unwrap_or_default()
            .as_bytes(),
        true,
    );

    let voltage = Sensor {
        name: "Gate battery voltage",
        unique_id: format!("{unique_id}_voltage"),
        device: device.clone(),
        state_topic: BATTERY_VOLTAGE_TOPIC,
        unit_of_measurement: "V",
        device_class: "voltage",
    };
    publish_raw(
        &format!("homeassistant/sensor/{unique_id}/voltage/config"),
        serde_json::to_string(&voltage)
            .unwrap_or_default()
            .as_bytes(),
        true,
    );

    let obstacle = BinarySensor {
        name: "Gate obstruction",
        unique_id: format!("{unique_id}_obstruction"),
        device: device.clone(),
        state_topic: OBSTACLE_TOPIC,
        device_class: "safety",
        payload_on: "on",
        payload_off: "off",
    };
    publish_raw(
        &format!("homeassistant/binary_sensor/{unique_id}/obstruction/config"),
        serde_json::to_string(&obstacle)
            .unwrap_or_default()
            .as_bytes(),
        true,
    );

    let fault = BinarySensor {
        name: "Gate fault",
        unique_id: format!("{unique_id}_fault"),
        device: device.clone(),
        state_topic: FAULT_TOPIC,
        device_class: "problem",
        payload_on: "on",
        payload_off: "off",
    };
    publish_raw(
        &format!("homeassistant/binary_sensor/{unique_id}/fault/config"),
        serde_json::to_string(&fault).unwrap_or_default().as_bytes(),
        true,
    );
}
