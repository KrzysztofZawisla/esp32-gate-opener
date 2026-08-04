pub const SSID: &str = env!("SSID");
pub const PASSWORD: &str = env!("PASSWORD");
pub const MQTT_BROKER: &str = env!("MQTT_BROKER");
pub const MQTT_USERNAME: &str = env!("MQTT_USERNAME");
pub const MQTT_PASSWORD: &str = env!("MQTT_PASSWORD");
pub const COMMAND_TOPIC: &str = env!("MQTT_COMMAND_TOPIC");
pub const STATUS_TOPIC: &str = env!("MQTT_STATUS_TOPIC");
pub const AVAILABILITY_TOPIC: &str = env!("MQTT_AVAILABILITY_TOPIC");
pub const BATTERY_TOPIC: &str = env!("MQTT_BATTERY_TOPIC");
pub const BATTERY_VOLTAGE_TOPIC: &str = env!("MQTT_BATTERY_VOLTAGE_TOPIC");
pub const OBSTACLE_TOPIC: &str = env!("MQTT_OBSTACLE_TOPIC");
pub const FAULT_TOPIC: &str = env!("MQTT_FAULT_TOPIC");

pub const LISTEN_PORT: u16 = 80;
pub const GATE_PULSE_MS: u64 = 1000;
pub const SENSOR_POLL_MS: u64 = 100;
pub const MQTT_KEEPALIVE_S: u64 = 10;
