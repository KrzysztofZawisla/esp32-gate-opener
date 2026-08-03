use esp_idf_svc::mqtt::client::{EspMqttEvent, EventPayload, QoS};
use log::{info, warn};

use crate::config::{AVAILABILITY_TOPIC, CMD_CLOSE, CMD_OPEN, COMMAND_TOPIC};
use crate::state;

use super::MQTT_CLIENT;
use super::publish_discovery;
use super::publish_fault;
use super::publish_obstacle;
use super::publish_raw;
use super::publish_status;

pub(crate) fn on_mqtt_event(event: EspMqttEvent) {
    match event.payload() {
        EventPayload::Connected(_) => {
            state::set_mqtt_connected(true);
            publish_raw(AVAILABILITY_TOPIC, b"online", true);
            publish_status();
            publish_obstacle();
            publish_fault();
            publish_discovery();
            if let Some(client) = MQTT_CLIENT.lock().unwrap().as_mut() {
                if let Err(e) = client.subscribe(COMMAND_TOPIC, QoS::AtMostOnce) {
                    warn!("MQTT subscribe to {COMMAND_TOPIC} failed: {e}");
                }
            }
        }
        EventPayload::Disconnected => {
            state::set_mqtt_connected(false);
        }
        EventPayload::Received {
            topic: Some(topic),
            data,
            ..
        } if topic == COMMAND_TOPIC => match data {
            b"open" => {
                info!("MQTT open command received");
                state::submit_command(CMD_OPEN);
            }
            b"close" => {
                info!("MQTT close command received");
                state::submit_command(CMD_CLOSE);
            }
            _ => {}
        },
        _ => {}
    }
}