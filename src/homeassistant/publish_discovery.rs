use crate::config::{
    AVAILABILITY_TOPIC, BATTERY_TOPIC, BATTERY_VOLTAGE_TOPIC, COMMAND_TOPIC, FAULT_TOPIC,
    OBSTACLE_TOPIC, STATUS_TOPIC,
};

use super::publish_raw;

pub(crate) fn publish_discovery() {
    let unique_id = "esp32_gate_opener";
    let device = format!(
        "{{\"identifiers\":[\"{unique_id}\"],\"name\":\"Gate\",\"manufacturer\":\"ESP32\",\"model\":\"Gate Opener\"}}"
    );

    let cover = format!(
        "{{\"name\":\"Gate\",\"unique_id\":\"{unique_id}_cover\",\"device\":{device},\"command_topic\":\"{COMMAND_TOPIC}\",\"state_topic\":\"{STATUS_TOPIC}\",\"availability_topic\":\"{AVAILABILITY_TOPIC}\",\"payload_open\":\"open\",\"payload_close\":\"close\",\"device_class\":\"gate\"}}"
    );
    publish_raw(&format!("homeassistant/cover/{unique_id}/config"), cover.as_bytes(), true);

    let battery = format!(
        "{{\"name\":\"Gate battery\",\"unique_id\":\"{unique_id}_battery\",\"device\":{device},\"state_topic\":\"{BATTERY_TOPIC}\",\"unit_of_measurement\":\"%\",\"device_class\":\"battery\"}}"
    );
    publish_raw(&format!("homeassistant/sensor/{unique_id}/battery/config"), battery.as_bytes(), true);

    let voltage = format!(
        "{{\"name\":\"Gate battery voltage\",\"unique_id\":\"{unique_id}_voltage\",\"device\":{device},\"state_topic\":\"{BATTERY_VOLTAGE_TOPIC}\",\"unit_of_measurement\":\"V\",\"device_class\":\"voltage\"}}"
    );
    publish_raw(&format!("homeassistant/sensor/{unique_id}/voltage/config"), voltage.as_bytes(), true);

    let obstacle = format!(
        "{{\"name\":\"Gate obstruction\",\"unique_id\":\"{unique_id}_obstruction\",\"device\":{device},\"state_topic\":\"{OBSTACLE_TOPIC}\",\"device_class\":\"safety\",\"payload_on\":\"on\",\"payload_off\":\"off\"}}"
    );
    publish_raw(&format!("homeassistant/binary_sensor/{unique_id}/obstruction/config"), obstacle.as_bytes(), true);

    let fault = format!(
        "{{\"name\":\"Gate fault\",\"unique_id\":\"{unique_id}_fault\",\"device\":{device},\"state_topic\":\"{FAULT_TOPIC}\",\"device_class\":\"problem\",\"payload_on\":\"on\",\"payload_off\":\"off\"}}"
    );
    publish_raw(&format!("homeassistant/binary_sensor/{unique_id}/fault/config"), fault.as_bytes(), true);
}