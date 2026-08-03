use super::*;

#[test]
fn sensor_status_mapping() {
    assert_eq!(sensor_status(true, false), ST_OPEN);
    assert_eq!(sensor_status(false, true), ST_CLOSED);
    assert_eq!(sensor_status(false, false), ST_STOPPED);
    assert_eq!(sensor_status(true, true), ST_ERROR);
}

#[test]
fn status_strings_round_trip() {
    assert_eq!(status_str(ST_OPEN), STATUS_OPEN);
    assert_eq!(status_str(ST_CLOSED), STATUS_CLOSED);
    assert_eq!(status_str(ST_OPENING), STATUS_OPENING);
    assert_eq!(status_str(ST_CLOSING), STATUS_CLOSING);
    assert_eq!(status_str(ST_ERROR), STATUS_ERROR);
    assert_eq!(status_str(99), STATUS_STOPPED);
}

#[test]
fn battery_percentage_linear_mapping() {
    assert_eq!(battery_pct_from_voltage_mv(12600.0, 12600.0, 11500.0), 100);
    assert_eq!(battery_pct_from_voltage_mv(11500.0, 12600.0, 11500.0), 0);
    assert_eq!(battery_pct_from_voltage_mv(12050.0, 12600.0, 11500.0), 50);
    assert_eq!(battery_pct_from_voltage_mv(13000.0, 12600.0, 11500.0), 100);
    assert_eq!(battery_pct_from_voltage_mv(11000.0, 12600.0, 11500.0), 0);
}

#[test]
fn battery_percentage_uses_median_and_divider() {
    let samples = [2000; 8];
    assert_eq!(median(&samples), Some(2000));
    assert_eq!(battery_pct_from_samples(&samples, 6.0, 12600.0, 11500.0), 45);
    assert_eq!(median(&[]), None);
    assert_eq!(battery_pct_from_samples(&[], 6.0, 12600.0, 11500.0), 0);
}

#[test]
fn obstacle_level_detection() {
    assert!(obstacle_blocked(true, true));
    assert!(!obstacle_blocked(false, true));
    assert!(obstacle_blocked(false, false));
    assert!(!obstacle_blocked(true, false));
}