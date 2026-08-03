pub const FAULT_BATTERY: u8 = 1 << 0;
pub const FAULT_SENSOR: u8 = 1 << 1;

pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_OPEN: &str = "open";
pub const STATUS_CLOSED: &str = "closed";
pub const STATUS_OPENING: &str = "opening";
pub const STATUS_CLOSING: &str = "closing";
pub const STATUS_ERROR: &str = "error";

pub const ST_STOPPED: u8 = 0;
pub const ST_OPEN: u8 = 1;
pub const ST_CLOSED: u8 = 2;
pub const ST_OPENING: u8 = 3;
pub const ST_CLOSING: u8 = 4;
pub const ST_ERROR: u8 = 5;

pub fn status_str(code: u8) -> &'static str {
    match code {
        ST_OPEN => STATUS_OPEN,
        ST_CLOSED => STATUS_CLOSED,
        ST_OPENING => STATUS_OPENING,
        ST_CLOSING => STATUS_CLOSING,
        ST_ERROR => STATUS_ERROR,
        _ => STATUS_STOPPED,
    }
}

pub fn sensor_status(open_active: bool, closed_active: bool) -> u8 {
    match (open_active, closed_active) {
        (true, true) => ST_ERROR,
        (true, false) => ST_OPEN,
        (false, true) => ST_CLOSED,
        (false, false) => ST_STOPPED,
    }
}

pub fn obstacle_blocked(pin_is_high: bool, active_high: bool) -> bool {
    pin_is_high == active_high
}

pub fn median(samples: &[u16]) -> Option<u16> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    Some(sorted[sorted.len() / 2])
}

pub fn battery_pct_from_voltage_mv(voltage_mv: f32, full_mv: f32, empty_mv: f32) -> u8 {
    if full_mv <= empty_mv {
        return 0;
    }
    let pct = (voltage_mv - empty_mv) / (full_mv - empty_mv) * 100.0;
    pct.clamp(0.0, 100.0) as u8
}

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

pub fn battery_divider_ratio() -> f32 {
    option_env!("BATTERY_DIVIDER_RATIO")
        .and_then(|v| v.parse().ok())
        .unwrap_or(6.0)
}

pub fn battery_full_mv() -> f32 {
    option_env!("BATTERY_FULL_MV")
        .and_then(|v| v.parse().ok())
        .unwrap_or(12600.0)
}

pub fn battery_empty_mv() -> f32 {
    option_env!("BATTERY_EMPTY_MV")
        .and_then(|v| v.parse().ok())
        .unwrap_or(11500.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_status_mapping() {
        assert_eq!(sensor_status(true, false), ST_OPEN);
        assert_eq!(sensor_status(false, true), ST_CLOSED);
        assert_eq!(sensor_status(false, false), ST_STOPPED);
        assert_eq!(sensor_status(true, true), ST_ERROR);
    }

    #[test]
    fn status_strings_round_trip() {
        assert_eq!(status_str(ST_OPEN), STATUS_OPEN);
        assert_eq!(status_str(ST_CLOSED), STATUS_CLOSED);
        assert_eq!(status_str(ST_OPENING), STATUS_OPENING);
        assert_eq!(status_str(ST_CLOSING), STATUS_CLOSING);
        assert_eq!(status_str(ST_ERROR), STATUS_ERROR);
        assert_eq!(status_str(99), STATUS_STOPPED);
    }

    #[test]
    fn battery_percentage_linear_mapping() {
        assert_eq!(battery_pct_from_voltage_mv(12600.0, 12600.0, 11500.0), 100);
        assert_eq!(battery_pct_from_voltage_mv(11500.0, 12600.0, 11500.0), 0);
        assert_eq!(battery_pct_from_voltage_mv(12050.0, 12600.0, 11500.0), 50);
        assert_eq!(battery_pct_from_voltage_mv(13000.0, 12600.0, 11500.0), 100);
        assert_eq!(battery_pct_from_voltage_mv(11000.0, 12600.0, 11500.0), 0);
    }

    #[test]
    fn battery_percentage_uses_median_and_divider() {
        let samples = [2000; 8];
        assert_eq!(median(&samples), Some(2000));
        assert_eq!(battery_pct_from_samples(&samples, 6.0, 12600.0, 11500.0), 45);
        assert_eq!(median(&[]), None);
        assert_eq!(battery_pct_from_samples(&[], 6.0, 12600.0, 11500.0), 0);
    }

    #[test]
    fn obstacle_level_detection() {
        assert!(obstacle_blocked(true, true));
        assert!(!obstacle_blocked(false, true));
        assert!(obstacle_blocked(false, false));
        assert!(!obstacle_blocked(true, false));
    }
}
