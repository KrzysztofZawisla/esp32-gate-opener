use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Device {
    #[serde(rename = "identifiers")]
    pub id: [&'static str; 1],
    pub name: &'static str,
    pub manufacturer: &'static str,
    pub model: &'static str,
}

#[derive(Serialize)]
pub struct Cover {
    pub name: &'static str,
    pub unique_id: String,
    pub device: Device,
    pub command_topic: &'static str,
    pub state_topic: &'static str,
    pub availability_topic: &'static str,
    pub payload_open: &'static str,
    pub payload_close: &'static str,
    pub device_class: &'static str,
}

#[derive(Serialize)]
pub struct Sensor {
    pub name: &'static str,
    pub unique_id: String,
    pub device: Device,
    pub state_topic: &'static str,
    pub unit_of_measurement: &'static str,
    pub device_class: &'static str,
}

#[derive(Serialize)]
pub struct BinarySensor {
    pub name: &'static str,
    pub unique_id: String,
    pub device: Device,
    pub state_topic: &'static str,
    pub device_class: &'static str,
    pub payload_on: &'static str,
    pub payload_off: &'static str,
}

pub struct DiscoveryTopics {
    pub command: &'static str,
    pub status: &'static str,
    pub availability: &'static str,
    pub battery: &'static str,
    pub battery_voltage: &'static str,
    pub obstacle: &'static str,
    pub fault: &'static str,
}

pub struct DiscoveryConfig {
    pub topic: String,
    pub payload: String,
}

pub fn discovery_configs(
    unique_id: &'static str,
    topics: &DiscoveryTopics,
) -> Vec<DiscoveryConfig> {
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
        command_topic: topics.command,
        state_topic: topics.status,
        availability_topic: topics.availability,
        payload_open: "open",
        payload_close: "close",
        device_class: "gate",
    };
    let battery = Sensor {
        name: "Gate battery",
        unique_id: format!("{unique_id}_battery"),
        device: device.clone(),
        state_topic: topics.battery,
        unit_of_measurement: "%",
        device_class: "battery",
    };
    let voltage = Sensor {
        name: "Gate battery voltage",
        unique_id: format!("{unique_id}_voltage"),
        device: device.clone(),
        state_topic: topics.battery_voltage,
        unit_of_measurement: "V",
        device_class: "voltage",
    };
    let obstacle = BinarySensor {
        name: "Gate obstruction",
        unique_id: format!("{unique_id}_obstruction"),
        device: device.clone(),
        state_topic: topics.obstacle,
        device_class: "safety",
        payload_on: "on",
        payload_off: "off",
    };
    let fault = BinarySensor {
        name: "Gate fault",
        unique_id: format!("{unique_id}_fault"),
        device,
        state_topic: topics.fault,
        device_class: "problem",
        payload_on: "on",
        payload_off: "off",
    };

    vec![
        (
            format!("homeassistant/cover/{unique_id}/config"),
            serde_json::to_string(&cover).unwrap_or_default(),
        ),
        (
            format!("homeassistant/sensor/{unique_id}/battery/config"),
            serde_json::to_string(&battery).unwrap_or_default(),
        ),
        (
            format!("homeassistant/sensor/{unique_id}/voltage/config"),
            serde_json::to_string(&voltage).unwrap_or_default(),
        ),
        (
            format!("homeassistant/binary_sensor/{unique_id}/obstruction/config"),
            serde_json::to_string(&obstacle).unwrap_or_default(),
        ),
        (
            format!("homeassistant/binary_sensor/{unique_id}/fault/config"),
            serde_json::to_string(&fault).unwrap_or_default(),
        ),
    ]
    .into_iter()
    .map(|(topic, payload)| DiscoveryConfig { topic, payload })
    .collect()
}
