use esp_idf_svc::mqtt::client::{EspMqttClient, LwtConfiguration, MqttClientConfiguration, QoS};
use log::{info, warn};

use crate::config::{
    AVAILABILITY_TOPIC, MQTT_BROKER, MQTT_KEEPALIVE_S, MQTT_PASSWORD, MQTT_USERNAME,
};

use super::non_empty;
use super::on_mqtt_event;
use super::MQTT_CLIENT;

pub fn connect_mqtt() {
    let username = non_empty(MQTT_USERNAME);
    let password = non_empty(MQTT_PASSWORD);
    let configuration = MqttClientConfiguration {
        client_id: Some("esp32-gate-opener"),
        keep_alive_interval: Some(core::time::Duration::from_secs(MQTT_KEEPALIVE_S)),
        reconnect_timeout: Some(core::time::Duration::from_secs(10)),
        network_timeout: core::time::Duration::from_secs(5),
        username,
        password,
        lwt: Some(LwtConfiguration {
            topic: AVAILABILITY_TOPIC,
            payload: b"offline",
            qos: QoS::AtLeastOnce,
            retain: true,
        }),
        ..Default::default()
    };

    match EspMqttClient::new_cb(MQTT_BROKER, &configuration, on_mqtt_event) {
        Ok(client) => {
            *MQTT_CLIENT.lock().unwrap() = Some(client);
            info!("MQTT client started");
        }
        Err(error) => warn!("Failed to start MQTT client: {error}"),
    }
}
