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