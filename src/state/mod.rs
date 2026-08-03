use core::sync::atomic::{AtomicBool, AtomicU8};

use crate::config::CMD_NONE;
use crate::pure::ST_STOPPED;

pub(crate) static STATUS_CODE: AtomicU8 = AtomicU8::new(ST_STOPPED);
pub(crate) static COMMAND: AtomicU8 = AtomicU8::new(CMD_NONE);
pub(crate) static FAULT: AtomicU8 = AtomicU8::new(0);
pub(crate) static OBSTACLE: AtomicBool = AtomicBool::new(false);
pub(crate) static BATTERY_PCT: AtomicU8 = AtomicU8::new(100);
pub(crate) static MQTT_CONNECTED: AtomicBool = AtomicBool::new(false);
pub(crate) static BATTERY_PUBLISH_REQUEST: AtomicBool = AtomicBool::new(false);

mod battery_pct;
mod clear_fault;
mod fault;
mod mqtt_connected;
mod obstacle;
#[cfg(target_os = "espidf")]
mod refresh_status;
mod request_battery_publish;
mod set_battery_pct;
mod set_fault;
mod set_mqtt_connected;
mod set_obstacle;
mod set_status_code;
mod status;
mod submit_command;
mod take_battery_publish_request;
mod take_command;

pub use battery_pct::battery_pct;
pub use clear_fault::clear_fault;
pub use fault::fault;
pub use mqtt_connected::mqtt_connected;
pub use obstacle::obstacle;
#[cfg(target_os = "espidf")]
pub use refresh_status::refresh_status;
pub use request_battery_publish::request_battery_publish;
pub use set_battery_pct::set_battery_pct;
pub use set_fault::set_fault;
pub use set_mqtt_connected::set_mqtt_connected;
pub use set_obstacle::set_obstacle;
pub use set_status_code::set_status_code;
pub use status::status;
pub use submit_command::submit_command;
pub use take_battery_publish_request::take_battery_publish_request;
pub use take_command::take_command;

#[cfg(test)]
mod tests;