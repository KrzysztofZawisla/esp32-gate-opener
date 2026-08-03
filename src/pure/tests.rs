use super::*;

#[test]
fn sensor_status_mapping() {
    assert_eq!(sensor_status(true, false), Status::Open);
    assert_eq!(sensor_status(false, true), Status::Closed);
    assert_eq!(sensor_status(false, false), Status::Stopped);
    assert_eq!(sensor_status(true, true), Status::Error);
}

#[test]
fn status_round_trip_and_strings() {
    for status in [
        Status::Stopped,
        Status::Open,
        Status::Closed,
        Status::Opening,
        Status::Closing,
        Status::Error,
    ] {
        assert_eq!(Status::from_raw(status.bits()), status);
    }
    assert_eq!(Status::Open.as_str(), "open");
    assert_eq!(Status::Closed.as_str(), "closed");
    assert_eq!(Status::Opening.as_str(), "opening");
    assert_eq!(Status::Closing.as_str(), "closing");
    assert_eq!(Status::Error.as_str(), "error");
    assert_eq!(Status::Stopped.as_str(), "stopped");
    assert_eq!(Status::from_raw(99), Status::Stopped);
}

#[test]
fn command_round_trip() {
    for command in [Command::None, Command::Open, Command::Close] {
        assert_eq!(Command::from_raw(command.bits()), command);
    }
    assert_eq!(Command::from_raw(99), Command::None);
}

#[test]
fn fault_is_a_bitmask() {
    assert_eq!((Fault::BATTERY | Fault::SENSOR).bits(), 0b11);
    assert!(Fault::BATTERY.contains(Fault::BATTERY));
    assert!(!Fault::BATTERY.contains(Fault::SENSOR));
    assert!((Fault::BATTERY | Fault::SENSOR).intersects(Fault::SENSOR));
    assert!(Fault::from_bits_retain(0).is_empty());
    assert_eq!(
        (Fault::BATTERY | Fault::SENSOR) & !Fault::BATTERY,
        Fault::SENSOR
    );
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
    assert_eq!(
        battery_pct_from_samples(&samples, 6.0, 12600.0, 11500.0),
        45
    );
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

#[test]
fn config_validation_ranges() {
    assert!(valid_battery_min_pct(0));
    assert!(valid_battery_min_pct(100));
    assert!(!valid_battery_min_pct(101));
    assert!(valid_grace_ms(0));
    assert!(valid_grace_ms(60_000));
    assert!(!valid_grace_ms(60_001));
    assert!(valid_motion_timeout_s(1));
    assert!(valid_motion_timeout_s(300));
    assert!(!valid_motion_timeout_s(0));
    assert!(!valid_motion_timeout_s(301));
    assert!(valid_gate_pulse_ms(1));
    assert!(valid_gate_pulse_ms(60_000));
    assert!(!valid_gate_pulse_ms(0));
    assert!(!valid_gate_pulse_ms(60_001));
    assert!(valid_telemetry_interval_s(1));
    assert!(valid_telemetry_interval_s(3600));
    assert!(!valid_telemetry_interval_s(0));
    assert!(!valid_telemetry_interval_s(3601));
}

#[test]
fn constant_time_eq_compares_in_constant_time_style() {
    assert!(constant_time_eq(b"abc", b"abc"));
    assert!(!constant_time_eq(b"abc", b"abd"));
    assert!(!constant_time_eq(b"abc", b"abcd"));
    assert!(constant_time_eq(b"", b""));
}

#[test]
fn parse_config_query_extracts_key_value_pairs() {
    let pairs = parse_config_query("battery_min_pct=100&grace_ms=2000").unwrap();
    assert_eq!(
        pairs,
        vec![
            ("battery_min_pct".to_string(), "100".to_string()),
            ("grace_ms".to_string(), "2000".to_string()),
        ]
    );
}

#[test]
fn parse_config_query_handles_empty_and_reset_flag() {
    assert_eq!(
        parse_config_query("").unwrap(),
        Vec::<(String, String)>::new()
    );

    let reset_on = parse_config_query("reset=1").unwrap();
    assert!(reset_on.iter().any(|(k, v)| k == "reset" && v == "1"));

    let reset_off = parse_config_query("battery_min_pct=10").unwrap();
    assert!(!reset_off.iter().any(|(k, v)| k == "reset" && v == "1"));
}

#[test]
fn parse_config_query_preserves_duplicate_keys_in_order() {
    let pairs = parse_config_query("a=1&a=2").unwrap();
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0], ("a".to_string(), "1".to_string()));
    assert_eq!(pairs[1], ("a".to_string(), "2".to_string()));
}

#[test]
fn parse_config_query_decodes_percent_encoding_leniently() {
    let pairs = parse_config_query("name=comt%C3%A9").unwrap();
    assert_eq!(pairs, vec![("name".to_string(), "comté".to_string())]);
    assert!(parse_config_query("a=%zz").is_ok());
    assert!(parse_config_query("a=%C3%28").is_ok());
    assert_eq!(parse_config_query("a").unwrap().len(), 1);
}

#[test]
fn discovery_configs_are_valid_homeassistant_payloads() {
    let topics = DiscoveryTopics {
        command: "cmd/topic",
        status: "status/topic",
        availability: "avail/topic",
        battery: "battery/topic",
        battery_voltage: "voltage/topic",
        obstacle: "obstacle/topic",
        fault: "fault/topic",
    };
    let configs = discovery_configs("gate1", &topics);

    assert_eq!(configs.len(), 5);
    assert_eq!(configs[0].topic, "homeassistant/cover/gate1/config");

    let cover: serde_json::Value = serde_json::from_str(&configs[0].payload).unwrap();
    assert_eq!(cover["name"], "Gate");
    assert_eq!(cover["unique_id"], "gate1_cover");
    assert_eq!(cover["command_topic"], "cmd/topic");
    assert_eq!(cover["state_topic"], "status/topic");
    assert_eq!(cover["availability_topic"], "avail/topic");
    assert_eq!(cover["payload_open"], "open");
    assert_eq!(cover["payload_close"], "close");
    assert_eq!(cover["device_class"], "gate");
    assert_eq!(cover["device"]["identifiers"][0], "gate1");
    assert_eq!(cover["device"]["name"], "Gate");
    assert_eq!(cover["device"]["model"], "Gate Opener");

    let battery: serde_json::Value = serde_json::from_str(&configs[1].payload).unwrap();
    assert_eq!(battery["unique_id"], "gate1_battery");
    assert_eq!(battery["state_topic"], "battery/topic");
    assert_eq!(battery["unit_of_measurement"], "%");
    assert_eq!(battery["device_class"], "battery");

    let voltage: serde_json::Value = serde_json::from_str(&configs[2].payload).unwrap();
    assert_eq!(voltage["unique_id"], "gate1_voltage");
    assert_eq!(voltage["state_topic"], "voltage/topic");
    assert_eq!(voltage["unit_of_measurement"], "V");
    assert_eq!(voltage["device_class"], "voltage");

    let obstacle: serde_json::Value = serde_json::from_str(&configs[3].payload).unwrap();
    assert_eq!(obstacle["unique_id"], "gate1_obstruction");
    assert_eq!(obstacle["state_topic"], "obstacle/topic");
    assert_eq!(obstacle["device_class"], "safety");
    assert_eq!(obstacle["payload_on"], "on");
    assert_eq!(obstacle["payload_off"], "off");

    let fault: serde_json::Value = serde_json::from_str(&configs[4].payload).unwrap();
    assert_eq!(fault["unique_id"], "gate1_fault");
    assert_eq!(fault["state_topic"], "fault/topic");
    assert_eq!(fault["device_class"], "problem");
}
