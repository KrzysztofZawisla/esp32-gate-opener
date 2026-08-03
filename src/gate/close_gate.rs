use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};
use log::warn;

use crate::config::{CMD_NONE, SENSOR_POLL_MS};
use crate::pure;
use crate::state;

use super::GatePins;
use super::grace_ms;
use super::motion_timeout_ms;
use super::pulse_interruptible;
use super::pulse_ms;
use super::reverse_to_open;
use super::set_lamp;
use super::wait_interruptible;

pub async fn close_gate(pins: &mut GatePins) -> Result<u8> {
    if pins.closed_sensor.is_low() {
        state::set_status_code(pure::ST_CLOSED);
        set_lamp(pins, false, false)?;
        return Ok(CMD_NONE);
    }

    if pure::obstacle_blocked(pins.obstacle_sensor.is_high(), pins.obstacle_active_level) {
        state::set_obstacle(true);
        warn!("Obstacle blocks the driveway, refusing to close");
        state::set_status_code(pure::ST_STOPPED);
        set_lamp(pins, false, false)?;
        return Ok(CMD_NONE);
    }

    state::set_status_code(pure::ST_CLOSING);
    set_lamp(pins, false, true)?;

    if let Some(cmd) = pulse_interruptible(&mut pins.close_relay, pulse_ms()).await? {
        return Ok(cmd);
    }
    if let Some(cmd) = wait_interruptible(grace_ms()).await? {
        return Ok(cmd);
    }

    let timeout_ms = motion_timeout_ms();
    let mut elapsed = 0u64;
    loop {
        if pins.closed_sensor.is_low() {
            state::set_obstacle(false);
            state::set_status_code(pure::ST_CLOSED);
            set_lamp(pins, false, false)?;
            return Ok(CMD_NONE);
        }
        if pure::obstacle_blocked(pins.obstacle_sensor.is_high(), pins.obstacle_active_level) {
            state::set_obstacle(true);
            warn!("Obstacle during closing, reversing to open");
            return reverse_to_open(pins).await;
        }
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            return Ok(cmd);
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
        if elapsed >= timeout_ms {
            warn!("Close timeout, reversing to open (fail-open)");
            return reverse_to_open(pins).await;
        }
    }
}