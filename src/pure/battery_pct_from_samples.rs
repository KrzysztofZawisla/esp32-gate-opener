use super::{battery_pct_from_voltage_mv, median};

pub fn battery_pct_from_samples(
    samples: &[u16],
    divider_ratio: f32,
    full_mv: f32,
    empty_mv: f32,
) -> u8 {
    match median(samples) {
        Some(pin_mv) => {
            battery_pct_from_voltage_mv(pin_mv as f32 * divider_ratio, full_mv, empty_mv)
        }
        None => 0,
    }
}
