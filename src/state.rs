use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::config::CMD_NONE;
use crate::pure::{status_str, ST_STOPPED};

#[cfg(target_os = "espidf")]
use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
#[cfg(target_os = "espidf")]
use crate::pure::{sensor_status, FAULT_SENSOR, ST_ERROR};

static STATUS_CODE: AtomicU8 = AtomicU8::new(ST_STOPPED);
static COMMAND: AtomicU8 = AtomicU8::new(CMD_NONE);
static FAULT: AtomicU8 = AtomicU8::new(0);
static OBSTACLE: AtomicBool = AtomicBool::new(false);
static BATTERY_PCT: AtomicU8 = AtomicU8::new(100);
static MQTT_CONNECTED: AtomicBool = AtomicBool::new(false);
static BATTERY_PUBLISH_REQUEST: AtomicBool = AtomicBool::new(false);

pub fn status() -> &'static str {
    status_str(STATUS_CODE.load(Ordering::Relaxed))
}

pub fn set_status_code(code: u8) {
    STATUS_CODE.store(code, Ordering::Relaxed);
}

#[cfg(target_os = "espidf")]
pub fn refresh_status(
    open_sensor: &PinDriver<'static, AnyIOPin, Input>,
    closed_sensor: &PinDriver<'static, AnyIOPin, Input>,
) {
    let code = sensor_status(open_sensor.is_low(), closed_sensor.is_low());
    STATUS_CODE.store(code, Ordering::Relaxed);
    if code == ST_ERROR {
        FAULT.fetch_or(FAULT_SENSOR, Ordering::Relaxed);
    } else {
        FAULT.fetch_and(!FAULT_SENSOR, Ordering::Relaxed);
    }
}

pub fn submit_command(cmd: u8) {
    COMMAND.store(cmd, Ordering::Relaxed);
}

pub fn take_command() -> u8 {
    COMMAND.swap(CMD_NONE, Ordering::Relaxed)
}

pub fn set_fault(mask: u8) {
    FAULT.store(mask, Ordering::Relaxed);
}

pub fn clear_fault(mask: u8) {
    FAULT.fetch_and(!mask, Ordering::Relaxed);
}

pub fn fault() -> u8 {
    FAULT.load(Ordering::Relaxed)
}

pub fn set_obstacle(on: bool) {
    OBSTACLE.store(on, Ordering::Relaxed);
}

pub fn obstacle() -> bool {
    OBSTACLE.load(Ordering::Relaxed)
}

pub fn set_battery_pct(pct: u8) {
    BATTERY_PCT.store(pct, Ordering::Relaxed);
}

pub fn battery_pct() -> u8 {
    BATTERY_PCT.load(Ordering::Relaxed)
}

pub fn set_mqtt_connected(connected: bool) {
    MQTT_CONNECTED.store(connected, Ordering::Relaxed);
}

pub fn mqtt_connected() -> bool {
    MQTT_CONNECTED.load(Ordering::Relaxed)
}

pub fn request_battery_publish() {
    BATTERY_PUBLISH_REQUEST.store(true, Ordering::Relaxed);
}

pub fn take_battery_publish_request() -> bool {
    BATTERY_PUBLISH_REQUEST.swap(false, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CMD_CLOSE, CMD_NONE, CMD_OPEN};
    use crate::pure::{FAULT_BATTERY, FAULT_SENSOR, STATUS_ERROR, ST_ERROR};

    #[test]
    fn command_slot_is_single_value_last_wins() {
        submit_command(CMD_OPEN);
        submit_command(CMD_CLOSE);
        assert_eq!(take_command(), CMD_CLOSE);
        assert_eq!(take_command(), CMD_NONE);
    }

    #[test]
    fn fault_is_a_replaceable_mask() {
        set_fault(0);
        set_fault(FAULT_BATTERY);
        assert_eq!(fault(), FAULT_BATTERY);
        set_fault(fault() & !FAULT_BATTERY);
        assert_eq!(fault(), 0);
    }

    #[test]
    fn clear_fault_only_clears_requested_bits() {
        set_fault(FAULT_BATTERY | FAULT_SENSOR);
        clear_fault(FAULT_BATTERY);
        assert_eq!(fault(), FAULT_SENSOR);
        clear_fault(FAULT_SENSOR);
        assert_eq!(fault(), 0);
    }

    #[test]
    fn battery_publish_request_is_one_shot() {
        request_battery_publish();
        assert!(take_battery_publish_request());
        assert!(!take_battery_publish_request());
    }

    #[test]
    fn sensor_error_status_maps_to_error_string() {
        set_status_code(ST_ERROR);
        assert_eq!(status(), STATUS_ERROR);
        assert_eq!(ST_ERROR, 5);
    }
}
