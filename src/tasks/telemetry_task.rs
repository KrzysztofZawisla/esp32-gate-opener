use core::borrow::Borrow;

use embassy_time::Duration as TimeDuration;
use embassy_time::{Instant, Timer};
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::gpio::ADCPin;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::warn;

use crate::config::SENSOR_POLL_MS;
use crate::config_storage;
use crate::homeassistant;
use crate::state;

pub(crate) async fn telemetry_task<AdcPin, AdcModule>(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    battery_channel: &mut AdcChannelDriver<'static, AdcPin, AdcModule>,
) where
    AdcPin: ADCPin,
    AdcModule: Borrow<AdcDriver<'static, AdcPin::Adc>>,
{
    const RECONNECT_INTERVAL_S: u64 = 5;
    let mut last_periodic = Instant::now();
    let mut last_reconnect = Instant::now();
    loop {
        match wifi.is_connected() {
            Ok(true) => {}
            _ => {
                if Instant::now().saturating_duration_since(last_reconnect)
                    >= TimeDuration::from_secs(RECONNECT_INTERVAL_S)
                {
                    last_reconnect = Instant::now();
                    warn!("WiFi connection lost, reconnecting");
                    let _ = wifi.disconnect();
                    let _ = wifi.connect();
                    let _ = wifi.wait_netif_up();
                }
            }
        }

        if state::mqtt_connected() {
            let interval = TimeDuration::from_secs(config_storage::telemetry_interval_s());
            let periodic_due = Instant::now().saturating_duration_since(last_periodic) >= interval;
            if periodic_due {
                last_periodic = Instant::now();
            }
            if state::take_battery_publish_request() || periodic_due {
                homeassistant::publish_battery(battery_channel);
            }
        }

        Timer::after(TimeDuration::from_millis(SENSOR_POLL_MS)).await;
    }
}