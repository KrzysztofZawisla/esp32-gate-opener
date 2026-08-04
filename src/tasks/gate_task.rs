use embassy_time::Duration as TimeDuration;
use embassy_time::Timer;
use log::warn;

use crate::config::SENSOR_POLL_MS;
use crate::config_storage;
use crate::gate::{self, GatePins};
use crate::homeassistant;
use crate::pure::{Command, Fault};
use crate::state;

pub(crate) async fn gate_task(pins: &mut GatePins) {
    loop {
        state::refresh_status(&pins.open_sensor, &pins.closed_sensor);
        let command = state::take_command();
        if command != Command::None {
            if state::battery_pct() < config_storage::battery_min_pct() {
                warn!(
                    "Battery too low ({}%), refusing to move the gate",
                    state::battery_pct()
                );
                state::set_fault(Fault::BATTERY);
                homeassistant::publish_fault();
                continue;
            }
            state::clear_fault(Fault::BATTERY);
            let mut current = command;
            loop {
                current = gate::handle_command(current, pins)
                    .await
                    .unwrap_or(Command::None);
                state::refresh_status(&pins.open_sensor, &pins.closed_sensor);
                homeassistant::publish_obstacle();
                if current == Command::None {
                    homeassistant::publish_status();
                    homeassistant::publish_fault();
                    break;
                }
            }
        }
        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
    }
}
