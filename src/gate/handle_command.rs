use anyhow::Result;

use crate::pure::Command;
use crate::state;

use super::close_gate;
use super::open_gate;
use super::GatePins;

pub async fn handle_command(command: Command, pins: &mut GatePins) -> Result<Command> {
    let result = match command {
        Command::Open => open_gate(pins).await,
        Command::Close => close_gate(pins).await,
        Command::None => Ok(Command::None),
    };

    if matches!(result, Ok(Command::None)) {
        state::request_battery_publish();
    }

    result
}
