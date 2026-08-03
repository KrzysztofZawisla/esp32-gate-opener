use crate::config_storage;
use crate::pure::{
    parse_config_query, valid_battery_min_pct, valid_gate_pulse_ms, valid_grace_ms,
    valid_motion_timeout_s, valid_telemetry_interval_s,
};
use anyhow::{anyhow, Result};
use esp_idf_svc::http::server::{EspHttpConnection, Request};
pub(crate) fn handle_config_update(request: &Request<&mut EspHttpConnection<'_>>) -> Result<()> {
    let uri = request.uri();
    let query = uri
        .split_once('?')
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    let parameters = parse_config_query(query)?;

    if parameters
        .iter()
        .any(|(key, value)| key == "reset" && value == "1")
    {
        config_storage::reset_all();
        return Ok(());
    }

    let mut failed = false;
    for (key, value) in parameters {
        match key.as_str() {
            "http_api_key" => {
                failed |= !config_storage::set_http_api_key(&value);
            }
            "battery_min_pct" => {
                if let Ok(parsed_value) = value.parse::<u8>() {
                    if valid_battery_min_pct(parsed_value) {
                        failed |= !config_storage::set_battery_min_pct(parsed_value);
                    }
                }
            }
            "grace_ms" => {
                if let Ok(parsed_value) = value.parse::<u16>() {
                    if valid_grace_ms(parsed_value) {
                        failed |= !config_storage::set_grace_ms(parsed_value);
                    }
                }
            }
            "motion_timeout_s" => {
                if let Ok(parsed_value) = value.parse::<u16>() {
                    if valid_motion_timeout_s(parsed_value) {
                        failed |= !config_storage::set_motion_timeout_s(parsed_value);
                    }
                }
            }
            "gate_pulse_ms" => {
                if let Ok(parsed_value) = value.parse::<u64>() {
                    if valid_gate_pulse_ms(parsed_value) {
                        failed |= !config_storage::set_gate_pulse_ms(parsed_value);
                    }
                }
            }
            "telemetry_interval_s" => {
                if let Ok(parsed_value) = value.parse::<u64>() {
                    if valid_telemetry_interval_s(parsed_value) {
                        failed |= !config_storage::set_telemetry_interval_s(parsed_value);
                    }
                }
            }
            _ => {}
        }
    }
    if failed {
        Err(anyhow!(
            "one or more config values were invalid or could not be persisted"
        ))
    } else {
        Ok(())
    }
}
