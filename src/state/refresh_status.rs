#[cfg(target_os = "espidf")]
use core::sync::atomic::Ordering;

#[cfg(target_os = "espidf")]
use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};

#[cfg(target_os = "espidf")]
use crate::pure::sensor_status;
#[cfg(target_os = "espidf")]
use crate::pure::{FAULT_SENSOR, ST_ERROR};
#[cfg(target_os = "espidf")]
use super::{FAULT, STATUS_CODE};

#[cfg(target_os = "espidf")]
pub fn refresh_status(
    open_sensor: &PinDriver<'static, AnyIOPin, Input>,
    closed_sensor: &PinDriver<'static, AnyIOPin, Input>,
) {
    let code = sensor_status(open_sensor.is_low(), closed_sensor.is_low());
    STATUS_CODE.store(code, Ordering::Relaxed);
    if code == ST_ERROR {
        FAULT.fetch_or(FAULT_SENSOR, Ordering::Relaxed);
    } else {
        FAULT.fetch_and(!FAULT_SENSOR, Ordering::Relaxed);
    }
}