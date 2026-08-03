use anyhow::{anyhow, Result};
use esp_idf_svc::http::server::{EspHttpConnection, Request};

use crate::config_storage;

pub(crate) fn handle_config_update(request: &Request<&mut EspHttpConnection<'_>>) -> Result<()> {
    let uri = request.uri();
    let query = uri
        .split_once('?')
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    let mut failed = false;
    for (key, value) in serde_urlencoded::from_str::<Vec<(String, String)>>(query)? {
        match key.as_str() {
            "http_api_key" => {
                failed |= !config_storage::set_http_api_key(&value);
            }
            "battery_min_pct" => {
                if let Ok(parsed_value) = value.parse::<u8>() {
                    if parsed_value <= 100 {
                        failed |= !config_storage::set_battery_min_pct(parsed_value);
                    }
                }
            }
            "grace_ms" => {
                if let Ok(parsed_value) = value.parse::<u16>() {
                    if parsed_value <= 60_000 {
                        failed |= !config_storage::set_grace_ms(parsed_value);
                    }
                }
            }
            "motion_timeout_s" => {
                if let Ok(parsed_value) = value.parse::<u16>() {
                    if (1..=300).contains(&parsed_value) {
                        failed |= !config_storage::set_motion_timeout_s(parsed_value);
                    }
                }
            }
            "gate_pulse_ms" => {
                if let Ok(parsed_value) = value.parse::<u64>() {
                    if (1..=60_000).contains(&parsed_value) {
                        failed |= !config_storage::set_gate_pulse_ms(parsed_value);
                    }
                }
            }
            "telemetry_interval_s" => {
                if let Ok(parsed_value) = value.parse::<u64>() {
                    if (1..=3600).contains(&parsed_value) {
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
