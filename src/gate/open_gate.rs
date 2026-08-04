use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};

use crate::config::SENSOR_POLL_MS;
use crate::pure::{Command, Status};
use crate::state;

use super::grace_ms;
use super::motion_timeout_ms;
use super::pulse_interruptible;
use super::pulse_ms;
use super::set_lamp;
use super::wait_interruptible;
use super::GatePins;
use log::{info, warn};

pub async fn open_gate(pins: &mut GatePins) -> Result<Command> {
    if pins.open_sensor.is_low() {
        state::set_status_code(Status::Open);
        set_lamp(pins, false, false)?;
        return Ok(Command::None);
    }

    state::set_status_code(Status::Opening);
    set_lamp(pins, true, false)?;

    let timeout_ms = motion_timeout_ms();
    for _attempt in 0..2 {
        if let Some(command) = pulse_interruptible(&mut pins.open_relay, pulse_ms()).await? {
            return Ok(command);
        }
        if let Some(command) = wait_interruptible(grace_ms()).await? {
            return Ok(command);
        }
        let mut elapsed = 0u64;
        loop {
            if pins.open_sensor.is_low() {
                break;
            }
            let command = state::take_command();
            if command != Command::None {
                return Ok(command);
            }
            Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
            elapsed += SENSOR_POLL_MS;
            if elapsed >= timeout_ms {
                break;
            }
        }
        if pins.open_sensor.is_low() {
            break;
        }
        info!("Open limit not reached in time, retrying the open pulse");
    }

    if pins.open_sensor.is_low() {
        state::set_status_code(Status::Open);
    } else {
        warn!("Open limit not reached, gate is not fully open");
        state::set_status_code(Status::Stopped);
    }
    set_lamp(pins, false, false)?;
    Ok(Command::None)
}
