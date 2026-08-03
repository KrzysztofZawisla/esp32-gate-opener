use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};

use crate::config::{CMD_NONE, SENSOR_POLL_MS};
use crate::pure;
use crate::state;

use super::grace_ms;
use super::motion_timeout_ms;
use super::pulse_interruptible;
use super::pulse_ms;
use super::set_lamp;
use super::wait_interruptible;
use super::GatePins;
use log::{info, warn};

pub async fn open_gate(pins: &mut GatePins) -> Result<u8> {
    if pins.open_sensor.is_low() {
        state::set_status_code(pure::ST_OPEN);
        set_lamp(pins, false, false)?;
        return Ok(CMD_NONE);
    }

    state::set_status_code(pure::ST_OPENING);
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
            if command != CMD_NONE {
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
        state::set_status_code(pure::ST_OPEN);
    } else {
        warn!("Open limit not reached, gate is not fully open");
        state::set_status_code(pure::ST_STOPPED);
    }
    set_lamp(pins, false, false)?;
    Ok(CMD_NONE)
}
