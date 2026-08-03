use anyhow::Result;

use crate::config::{CMD_CLOSE, CMD_NONE, CMD_OPEN};
use crate::state;

use super::GatePins;
use super::close_gate;
use super::open_gate;

pub async fn handle_command(command: u8, pins: &mut GatePins) -> Result<u8> {
    let result = match command {
        CMD_OPEN => open_gate(pins).await,
        CMD_CLOSE => close_gate(pins).await,
        _ => Ok(CMD_NONE),
    };

    if matches!(result, Ok(CMD_NONE)) {
        state::request_battery_publish();
    }

    result
}