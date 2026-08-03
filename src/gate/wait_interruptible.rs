use anyhow::Result;
use embassy_time::{Duration as TimeDuration, Timer};

use crate::config::SENSOR_POLL_MS;
use crate::pure::Command;
use crate::state;

pub async fn wait_interruptible(duration_ms: u64) -> Result<Option<Command>> {
    let mut elapsed = 0u64;
    while elapsed < duration_ms {
        let command = state::take_command();
        if command != Command::None {
            return Ok(Some(command));
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
        elapsed += SENSOR_POLL_MS;
    }
    Ok(None)
}
