use embedded_svc::io::Write;
use esp_idf_hal::io::EspIOError;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use log::{error, info};

use anyhow::Result;

use crate::config::{CMD_CLOSE, CMD_OPEN, LISTEN_PORT};
use crate::{config_storage, ota, state};

use super::check_auth;
use super::handle_config_update;

pub fn start_http_server() -> Result<EspHttpServer<'static>> {
    let configuration = HttpConfig {
        http_port: LISTEN_PORT,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&configuration)?;

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
            let result = handle_config_update(&req);
            if let Err(error) = &result {
                log::warn!("Failed to update config: {error}");
            }
            let _ = match result {
                Ok(()) => req.into_ok_response()?,
                Err(_) => req.into_status_response(500)?,
            };
        }
        Ok(())
    })?;

    server.fn_handler("/ota", Method::Post, |req| -> Result<(), EspIOError> {
        if !check_auth(&req) {
            return req.into_status_response(401).map(|_| ());
        }
        let mut request = req;
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
