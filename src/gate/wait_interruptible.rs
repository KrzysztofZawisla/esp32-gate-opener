use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};

use crate::config::{CMD_NONE, SENSOR_POLL_MS};
use crate::state;

pub async fn wait_interruptible(duration_ms: u64) -> Result<Option<u8>> {
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