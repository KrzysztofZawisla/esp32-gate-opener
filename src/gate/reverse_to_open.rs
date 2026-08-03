use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};

use crate::config::{CMD_NONE, SENSOR_POLL_MS};
use crate::pure;
use crate::state;

use super::GatePins;
use super::grace_ms;
use super::motion_timeout_ms;
use super::pulse_interruptible;
use super::pulse_ms;
use super::set_lamp;
use super::wait_interruptible;

pub async fn reverse_to_open(pins: &mut GatePins) -> Result<u8> {
    state::set_status_code(pure::ST_OPENING);
    set_lamp(pins, true, false)?;

    if let Some(cmd) = pulse_interruptible(&mut pins.open_relay, pulse_ms()).await? {
        return Ok(cmd);
    }
    if let Some(cmd) = wait_interruptible(grace_ms()).await? {
        return Ok(cmd);
    }

    let timeout_ms = motion_timeout_ms();
    let mut elapsed = 0u64;
    while elapsed < timeout_ms {
        if pins.open_sensor.is_low() {
            break;
        }
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            return Ok(cmd);
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
    }

    if pins.open_sensor.is_low() {
        state::set_status_code(pure::ST_OPEN);
    } else {
        state::set_status_code(pure::ST_STOPPED);
    }
    set_lamp(pins, false, false)?;
    Ok(CMD_NONE)
}