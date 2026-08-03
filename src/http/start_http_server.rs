use embedded_svc::io::Write;
use esp_idf_hal::io::EspIOError;
use esp_idf_hal::reset;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use log::{error, info};

use anyhow::Result;

use crate::config::LISTEN_PORT;
use crate::pure::Command;
use crate::{config_storage, ota, state};

use super::check_auth;
use super::handle_config_update;

pub fn start_http_server() -> Result<EspHttpServer<'static>> {
    let configuration = HttpConfig {
        http_port: LISTEN_PORT,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&configuration)?;

    server.fn_handler("/open", Method::Post, |request| -> Result<(), EspIOError> {
        if !check_auth(&request) {
            request.into_status_response(401)?;
        } else {
            info!("HTTP /open command received");
            state::submit_command(Command::Open);
            request.into_ok_response()?;
        }
        Ok(())
    })?;

    server.fn_handler(
        "/close",
        Method::Post,
        |request| -> Result<(), EspIOError> {
            if !check_auth(&request) {
                request.into_status_response(401)?;
            } else {
                info!("HTTP /close command received");
                state::submit_command(Command::Close);
                request.into_ok_response()?;
            }
            Ok(())
        },
    )?;

    server.fn_handler(
        "/status",
        Method::Get,
        |request| -> Result<(), EspIOError> {
            let body = state::status();
            let mut response =
                request.into_response(200, None, &[("Content-Type", "text/plain")])?;
            response.write_all(body.as_bytes())?;
            Ok(())
        },
    )?;

    server.fn_handler(
        "/config",
        Method::Get,
        |request| -> Result<(), EspIOError> {
            if !check_auth(&request) {
                request.into_status_response(401)?;
            } else {
                let stored_key = config_storage::http_api_key();
                let displayed_key = if stored_key.is_empty() {
                    String::new()
                } else {
                    "***".to_string()
                };
                let body = ConfigResponse {
                    http_api_key: displayed_key,
                    battery_min_pct: config_storage::battery_min_pct(),
                    grace_ms: config_storage::grace_ms(),
                    motion_timeout_s: config_storage::motion_timeout_s(),
                    gate_pulse_ms: config_storage::gate_pulse_ms(),
                    telemetry_interval_s: config_storage::telemetry_interval_s(),
                };
                let mut response =
                    request.into_response(200, None, &[("Content-Type", "application/json")])?;
                if let Ok(serialized) = serde_json::to_string(&body) {
                    response.write_all(serialized.as_bytes())?;
                }
            }
            Ok(())
        },
    )?;

    server.fn_handler(
        "/config",
        Method::Post,
        |request| -> Result<(), EspIOError> {
            if !check_auth(&request) {
                request.into_status_response(401)?;
            } else {
                let result = handle_config_update(&request);
                if let Err(error) = &result {
                    log::warn!("Failed to update config: {error}");
                }
                let _ = match result {
                    Ok(()) => request.into_ok_response()?,
                    Err(_) => request.into_status_response(500)?,
                };
            }
            Ok(())
        },
    )?;

    server.fn_handler(
        "/reboot",
        Method::Post,
        |request| -> Result<(), EspIOError> {
            if !check_auth(&request) {
                request.into_status_response(401)?;
            } else {
                info!("HTTP /reboot command received, restarting");
                reset::restart();
            }
            Ok(())
        },
    )?;

    server.fn_handler("/ota", Method::Post, |request| -> Result<(), EspIOError> {
        if !check_auth(&request) {
            return request.into_status_response(401).map(|_| ());
        }
        let mut request = request;
        let result = ota::flash_ota(&mut |output: &mut [u8]| request.read(output));
        match result {
            Ok(()) => {}
            Err(error) => {
                error!("OTA failed: {error}");
                let _ = request.into_status_response(500);
            }
        }
        Ok(())
    })?;

    Ok(server)
}

#[derive(serde::Serialize)]
struct ConfigResponse {
    http_api_key: String,
    battery_min_pct: u8,
    grace_ms: u16,
    motion_timeout_s: u16,
    gate_pulse_ms: u64,
    telemetry_interval_s: u64,
}
