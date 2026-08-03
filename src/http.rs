use embedded_svc::io::Write;
use esp_idf_hal::io::EspIOError;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use log::{error, info, warn};

use anyhow::{anyhow, Result};

use crate::config::{CMD_CLOSE, CMD_OPEN, LISTEN_PORT};
use crate::{config_storage, ota, state};

pub fn start_http_server() -> Result<EspHttpServer<'static>> {
    let config = HttpConfig {
        http_port: LISTEN_PORT,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    server.fn_handler("/open", Method::Post, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            req.into_status_response(401)?;
        } else {
            info!("HTTP /open command received");
            state::submit_command(CMD_OPEN);
            req.into_ok_response()?;
        }
        Ok(())
    })?;

    server.fn_handler("/close", Method::Post, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            req.into_status_response(401)?;
        } else {
            info!("HTTP /close command received");
            state::submit_command(CMD_CLOSE);
            req.into_ok_response()?;
        }
        Ok(())
    })?;

    server.fn_handler("/status", Method::Get, |req| -> Result<(), EspIOError> {
        let body = state::status();
        let mut resp = req.into_response(200, None, &[("Content-Type", "text/plain")])?;
        resp.write_all(body.as_bytes())?;
        Ok(())
    })?;

    server.fn_handler("/config", Method::Get, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            req.into_status_response(401)?;
        } else {
            let key = config_storage::http_api_key();
            let key = if key.is_empty() { String::new() } else { "***".to_string() };
            let body = format!(
                "{{\"http_api_key\":\"{}\",\"battery_min_pct\":{},\"grace_ms\":{},\"motion_timeout_s\":{},\"gate_pulse_ms\":{},\"telemetry_interval_s\":{}}}",
                key,
                config_storage::battery_min_pct(),
                config_storage::grace_ms(),
                config_storage::motion_timeout_s(),
                config_storage::gate_pulse_ms(),
                config_storage::telemetry_interval_s(),
            );
            let mut resp = req.into_response(200, None, &[("Content-Type", "application/json")])?;
            resp.write_all(body.as_bytes())?;
        }
        Ok(())
    })?;

    server.fn_handler("/config", Method::Post, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            req.into_status_response(401)?;
        } else {
            match handle_config_update(&req) {
                Ok(()) => req.into_ok_response()?,
                Err(e) => {
                    warn!("Failed to update config: {e}");
                    req.into_status_response(500)?;
                }
            }
        }
        Ok(())
    })?;

    server.fn_handler("/ota", Method::Post, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            return req.into_status_response(401).map(|_| ());
        }
        let mut req = req;
        let result = {
            ota::flash_ota(&mut |out: &mut [u8]| req.read(out))
        };
        match result {
            Ok(()) => {}
            Err(e) => {
                error!("OTA failed: {e}");
                let _ = req.into_status_response(500);
            }
        }
        Ok(())
    })?;

    Ok(server)
}

fn check_auth(req: &Request<&mut EspHttpConnection<'_>>) -> bool {
    let key = config_storage::http_api_key();
    if key.is_empty() {
        return true;
    }
    req.header("X-Api-Key") == Some(key.as_str())
}

fn handle_config_update(req: &Request<&mut EspHttpConnection<'_>>) -> Result<()> {
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

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
