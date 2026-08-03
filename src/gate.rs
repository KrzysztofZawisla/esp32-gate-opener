use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Instant, Timer};
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Input, Output, PinDriver};
use log::{info, warn};

use crate::config::{CMD_CLOSE, CMD_NONE, CMD_OPEN, SENSOR_POLL_MS};
use crate::config_storage;
use crate::pure;
use crate::state;

pub fn set_lamp(
    lamp_green: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_red: &mut PinDriver<'static, AnyOutputPin, Output>,
    green: bool,
    red: bool,
) -> Result<()> {
    if green {
        lamp_green.set_high()?;
        lamp_red.set_low()?;
    } else if red {
        lamp_red.set_high()?;
        lamp_green.set_low()?;
    } else {
        lamp_green.set_low()?;
        lamp_red.set_low()?;
    }
    Ok(())
}

pub async fn handle_command(
    command: u8,
    open_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    close_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_green: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_red: &mut PinDriver<'static, AnyOutputPin, Output>,
    open_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    closed_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    obstacle_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    obstacle_active_level: bool,
) -> Result<u8> {
    let result = match command {
        CMD_OPEN => {
            open_gate(open_relay, lamp_green, lamp_red, open_sensor).await
        }
        CMD_CLOSE => {
            close_gate(
                open_relay,
                close_relay,
                lamp_green,
                lamp_red,
                open_sensor,
                closed_sensor,
                obstacle_sensor,
                obstacle_active_level,
            )
            .await
        }
        _ => Ok(CMD_NONE),
    };

    if matches!(result, Ok(CMD_NONE)) {
        state::request_battery_publish();
    }

    result
}

async fn open_gate(
    open_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_green: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_red: &mut PinDriver<'static, AnyOutputPin, Output>,
    open_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
) -> Result<u8> {
    if open_sensor.is_low() {
        state::set_status_code(pure::ST_OPEN);
        set_lamp(lamp_green, lamp_red, false, false)?;
        return Ok(CMD_NONE);
    }

    state::set_status_code(pure::ST_OPENING);
    set_lamp(lamp_green, lamp_red, true, false)?;

    let timeout_ms = motion_timeout_ms();
    for _attempt in 0..2 {
        if let Some(cmd) = pulse_interruptible(open_relay, pulse_ms()).await? {
            return Ok(cmd);
        }
        if let Some(cmd) = wait_interruptible(grace_ms()).await? {
            return Ok(cmd);
        }
        let mut elapsed = 0u64;
        loop {
            if open_sensor.is_low() {
                break;
            }
            let cmd = state::take_command();
            if cmd != CMD_NONE {
                return Ok(cmd);
            }
            Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
            elapsed += SENSOR_POLL_MS;
            if elapsed >= timeout_ms {
                break;
            }
        }
        if open_sensor.is_low() {
            break;
        }
        info!("Open limit not reached in time, retrying the open pulse");
    }

    if open_sensor.is_low() {
        state::set_status_code(pure::ST_OPEN);
    } else {
        warn!("Open limit not reached, gate is not fully open");
        state::set_status_code(pure::ST_STOPPED);
    }
    set_lamp(lamp_green, lamp_red, false, false)?;
    Ok(CMD_NONE)
}

async fn close_gate(
    open_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    close_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_green: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_red: &mut PinDriver<'static, AnyOutputPin, Output>,
    open_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    closed_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    obstacle_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
    obstacle_active_level: bool,
) -> Result<u8> {
    if closed_sensor.is_low() {
        state::set_status_code(pure::ST_CLOSED);
        set_lamp(lamp_green, lamp_red, false, false)?;
        return Ok(CMD_NONE);
    }

    if pure::obstacle_blocked(obstacle_sensor.is_high(), obstacle_active_level) {
        state::set_obstacle(true);
        warn!("Obstacle blocks the driveway, refusing to close");
        state::set_status_code(pure::ST_STOPPED);
        set_lamp(lamp_green, lamp_red, false, false)?;
        return Ok(CMD_NONE);
    }

    state::set_status_code(pure::ST_CLOSING);
    set_lamp(lamp_green, lamp_red, false, true)?;

    if let Some(cmd) = pulse_interruptible(close_relay, pulse_ms()).await? {
        return Ok(cmd);
    }
    if let Some(cmd) = wait_interruptible(grace_ms()).await? {
        return Ok(cmd);
    }

    let timeout_ms = motion_timeout_ms();
    let mut elapsed = 0u64;
    loop {
        if closed_sensor.is_low() {
            state::set_obstacle(false);
            state::set_status_code(pure::ST_CLOSED);
            set_lamp(lamp_green, lamp_red, false, false)?;
            return Ok(CMD_NONE);
        }
        if pure::obstacle_blocked(obstacle_sensor.is_high(), obstacle_active_level) {
            state::set_obstacle(true);
            warn!("Obstacle during closing, reversing to open");
            return reverse_to_open(open_relay, lamp_green, lamp_red, open_sensor).await;
        }
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            return Ok(cmd);
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
        if elapsed >= timeout_ms {
            warn!("Close timeout, reversing to open (fail-open)");
            return reverse_to_open(open_relay, lamp_green, lamp_red, open_sensor).await;
        }
    }
}

async fn reverse_to_open(
    open_relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_green: &mut PinDriver<'static, AnyOutputPin, Output>,
    lamp_red: &mut PinDriver<'static, AnyOutputPin, Output>,
    open_sensor: &mut PinDriver<'static, AnyIOPin, Input>,
) -> Result<u8> {
    state::set_status_code(pure::ST_OPENING);
    set_lamp(lamp_green, lamp_red, true, false)?;

    if let Some(cmd) = pulse_interruptible(open_relay, pulse_ms()).await? {
        return Ok(cmd);
    }
    if let Some(cmd) = wait_interruptible(grace_ms()).await? {
        return Ok(cmd);
    }

    let timeout_ms = motion_timeout_ms();
    let mut elapsed = 0u64;
    while elapsed < timeout_ms {
        if open_sensor.is_low() {
            break;
        }
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            return Ok(cmd);
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
    }

    if open_sensor.is_low() {
        state::set_status_code(pure::ST_OPEN);
    } else {
        state::set_status_code(pure::ST_STOPPED);
    }
    set_lamp(lamp_green, lamp_red, false, false)?;
    Ok(CMD_NONE)
}

async fn pulse_interruptible(
    relay: &mut PinDriver<'static, AnyOutputPin, Output>,
    duration_ms: u64,
) -> Result<Option<u8>> {
    relay.set_high()?;
    let start = Instant::now();
    let duration = TimeDuration::from_millis(duration_ms);
    loop {
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            relay.set_low()?;
            return Ok(Some(cmd));
        }
        if Instant::now().saturating_duration_since(start) >= duration {
            break;
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
    }
    relay.set_low()?;
    Ok(None)
}

async fn wait_interruptible(duration_ms: u64) -> Result<Option<u8>> {
    let mut elapsed = 0u64;
    while elapsed < duration_ms {
        let cmd = state::take_command();
        if cmd != CMD_NONE {
            return Ok(Some(cmd));
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
    }
    Ok(None)
}

fn pulse_ms() -> u64 {
    config_storage::gate_pulse_ms()
}

fn grace_ms() -> u64 {
    config_storage::grace_ms() as u64
}

fn motion_timeout_ms() -> u64 {
    config_storage::motion_timeout_s() as u64 * 1000
}
