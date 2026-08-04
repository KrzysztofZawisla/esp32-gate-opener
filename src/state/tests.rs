use super::*;
use crate::pure::{Command, Fault, Status};

#[test]
fn command_slot_is_single_value_last_wins() {
    submit_command(Command::Open);
    submit_command(Command::Close);
    assert_eq!(take_command(), Command::Close);
    assert_eq!(take_command(), Command::None);
}

#[test]
fn fault_is_a_replaceable_mask() {
    set_fault(Fault::empty());
    set_fault(Fault::BATTERY);
    assert_eq!(fault(), Fault::BATTERY);
    set_fault(fault() & !Fault::BATTERY);
    assert_eq!(fault(), Fault::empty());
}

#[test]
fn clear_fault_only_clears_requested_bits() {
    set_fault(Fault::BATTERY | Fault::SENSOR);
    clear_fault(Fault::BATTERY);
    assert_eq!(fault(), Fault::SENSOR);
    clear_fault(Fault::SENSOR);
    assert_eq!(fault(), Fault::empty());
}

#[test]
fn battery_publish_request_is_one_shot() {
    request_battery_publish();
    assert!(take_battery_publish_request());
    assert!(!take_battery_publish_request());
}

#[test]
fn error_status_maps_to_error_string() {
    set_status_code(Status::Error);
    assert_eq!(status(), "error");
    assert_eq!(Status::Error.bits(), 5);
}
