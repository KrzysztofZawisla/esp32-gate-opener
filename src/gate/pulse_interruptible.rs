use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Instant, Timer};
use esp_idf_hal::gpio::{AnyOutputPin, Output, PinDriver};

use crate::config::{CMD_NONE, SENSOR_POLL_MS};
use crate::state;

pub async fn pulse_interruptible(
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