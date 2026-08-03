use anyhow::{anyhow, Result};
use esp_idf_svc::http::server::{EspHttpConnection, Request};

use crate::config_storage;

use super::url_decode;

pub(crate) fn handle_config_update(req: &Request<&mut EspHttpConnection<'_>>) -> Result<()> {
    let uri = req.uri();
    let Some(query) = uri.split_once('?').map(|(_, q)| q) else {
        return Ok(());
    };
    let mut failed = false;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let value = url_decode(value);
        match key {
            "http_api_key" => {
                failed |= !config_storage::set_http_api_key(&value);
            }
            "battery_min_pct" => {
                if let Ok(v) = value.parse::<u8>() {
                    if v <= 100 {
                        failed |= !config_storage::set_battery_min_pct(v);
                    }
                }
            }
            "grace_ms" => {
                if let Ok(v) = value.parse::<u16>() {
                    if v <= 60_000 {
                        failed |= !config_storage::set_grace_ms(v);
                    }
                }
            }
            "motion_timeout_s" => {
                if let Ok(v) = value.parse::<u16>() {
                    if (1..=300).contains(&v) {
                        failed |= !config_storage::set_motion_timeout_s(v);
                    }
                }
            }
            "gate_pulse_ms" => {
                if let Ok(v) = value.parse::<u64>() {
                    if (1..=60_000).contains(&v) {
                        failed |= !config_storage::set_gate_pulse_ms(v);
                    }
                }
            }
            "telemetry_interval_s" => {
                if let Ok(v) = value.parse::<u64>() {
                    if (1..=3600).contains(&v) {
                        failed |= !config_storage::set_telemetry_interval_s(v);
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