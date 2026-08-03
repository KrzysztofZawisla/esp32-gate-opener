use std::sync::Mutex;

use esp_idf_svc::mqtt::client::EspMqttClient;

pub(crate) static MQTT_CLIENT: Mutex<Option<EspMqttClient<'static>>> = Mutex::new(None);

mod connect_mqtt;
mod non_empty;
mod on_mqtt_event;
mod publish_battery;
mod publish_discovery;
mod publish_fault;
mod publish_obstacle;
mod publish_raw;
mod publish_status;

pub use connect_mqtt::connect_mqtt;
pub use publish_battery::publish_battery;
pub use publish_fault::publish_fault;
pub use publish_obstacle::publish_obstacle;
pub use publish_status::publish_status;

pub(crate) use non_empty::non_empty;
pub(crate) use on_mqtt_event::on_mqtt_event;
pub(crate) use publish_discovery::publish_discovery;
pub(crate) use publish_raw::publish_raw;