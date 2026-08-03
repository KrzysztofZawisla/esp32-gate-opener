use core::borrow::Borrow;
use std::sync::Mutex;

use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::gpio::ADCPin;
use esp_idf_svc::mqtt::client::{
    EspMqttClient, EspMqttEvent, EventPayload, LwtConfiguration, MqttClientConfiguration, QoS,
};
use log::{info, warn};

use crate::config::*;
use crate::{pure, state};

static MQTT_CLIENT: Mutex<Option<EspMqttClient<'static>>> = Mutex::new(None);

pub fn connect_mqtt() {
    let username = (!MQTT_USERNAME.is_empty()).then_some(MQTT_USERNAME);
    let password = (!MQTT_PASSWORD.is_empty()).then_some(MQTT_PASSWORD);
    let conf = MqttClientConfiguration {
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

    match EspMqttClient::new_cb(MQTT_BROKER, &conf, on_mqtt_event) {
        Ok(client) => {
            *MQTT_CLIENT.lock().unwrap() = Some(client);
            info!("MQTT client started");
        }
        Err(e) => warn!("Failed to start MQTT client: {e}"),
    }
}

fn on_mqtt_event(event: EspMqttEvent) {
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

pub fn publish_status() {
    publish_raw(STATUS_TOPIC, state::status().as_bytes(), true);
}

pub fn publish_obstacle() {
    let payload: &[u8] = if state::obstacle() { b"on" } else { b"off" };
    publish_raw(OBSTACLE_TOPIC, payload, true);
}

pub fn publish_fault() {
    let payload: &[u8] = if state::fault() != 0 { b"on" } else { b"off" };
    publish_raw(FAULT_TOPIC, payload, true);
}

pub fn publish_battery<C, M>(
    battery_channel: &mut AdcChannelDriver<'static, C, M>,
) where
    C: ADCPin,
    M: Borrow<AdcDriver<'static, C::Adc>>,
{
    let mut samples = [0u16; 8];
    let mut ok = true;
    for sample in samples.iter_mut() {
        match battery_channel.read() {
            Ok(v) => *sample = v,
            Err(e) => {
                warn!("ADC read failed: {e}");
                ok = false;
                break;
            }
        }
    }
    if !ok {
        state::set_battery_pct(0);
        publish_raw(BATTERY_TOPIC, b"0", true);
        return;
    }

    let pct = pure::battery_pct_from_samples(
        &samples,
        pure::battery_divider_ratio(),
        pure::battery_full_mv(),
        pure::battery_empty_mv(),
    );
    state::set_battery_pct(pct);

    publish_raw(BATTERY_TOPIC, pct.to_string().as_bytes(), true);

    if let Some(median_mv) = pure::median(&samples) {
        let voltage = median_mv as f32 * pure::battery_divider_ratio() / 1000.0;
        let payload = format!("{voltage:.2}");
        publish_raw(BATTERY_VOLTAGE_TOPIC, payload.as_bytes(), true);
    }
}

fn publish_discovery() {
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

fn publish_raw(topic: &str, payload: &[u8], retain: bool) -> bool {
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
