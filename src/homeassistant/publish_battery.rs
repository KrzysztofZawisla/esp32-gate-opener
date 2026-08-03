use core::borrow::Borrow;
use log::warn;

use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::gpio::ADCPin;

use crate::config::{BATTERY_TOPIC, BATTERY_VOLTAGE_TOPIC};
use crate::{pure, state};

use super::publish_raw;

pub fn publish_battery<AdcPin, AdcModule>(
    battery_channel: &mut AdcChannelDriver<'static, AdcPin, AdcModule>,
) where
    AdcPin: ADCPin,
    AdcModule: Borrow<AdcDriver<'static, AdcPin::Adc>>,
{
    let mut samples = [0u16; 8];
    let mut read_ok = true;
    for sample in samples.iter_mut() {
        match battery_channel.read() {
            Ok(value) => *sample = value,
            Err(error) => {
                warn!("ADC read failed: {error}");
                read_ok = false;
                break;
            }
        }
    }
    if !read_ok {
        state::set_battery_pct(0);
        publish_raw(BATTERY_TOPIC, b"0", true);
        return;
    }

    let percentage = pure::battery_pct_from_samples(
        &samples,
        pure::battery_divider_ratio(),
        pure::battery_full_mv(),
        pure::battery_empty_mv(),
    );
    state::set_battery_pct(percentage);

    publish_raw(BATTERY_TOPIC, percentage.to_string().as_bytes(), true);

    if let Some(median_mv) = pure::median(&samples) {
        let voltage = median_mv as f32 * pure::battery_divider_ratio() / 1000.0;
        let payload = format!("{voltage:.2}");
        publish_raw(BATTERY_VOLTAGE_TOPIC, payload.as_bytes(), true);
    }
}
