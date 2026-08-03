use esp_idf_svc::mqtt::client::QoS;
use log::warn;

use super::MQTT_CLIENT;

pub(crate) fn publish_raw(topic: &str, payload: &[u8], retain: bool) -> bool {
    let mut guard = MQTT_CLIENT.lock().unwrap();
    let Some(client) = guard.as_mut() else {
        return false;
    };
    match client.publish(topic, QoS::AtLeastOnce, retain, payload) {
        Ok(_) => true,
        Err(e) => {
            warn!("MQTT publish to {topic} failed: {e}");
            false
        }
    }
}