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

mod battery_divider_ratio;
mod battery_empty_mv;
mod battery_full_mv;
mod battery_pct_from_samples;
mod battery_pct_from_voltage_mv;
mod median;
mod obstacle_blocked;
mod sensor_status;
mod status_str;

pub use battery_divider_ratio::battery_divider_ratio;
pub use battery_empty_mv::battery_empty_mv;
pub use battery_full_mv::battery_full_mv;
pub use battery_pct_from_samples::battery_pct_from_samples;
pub use battery_pct_from_voltage_mv::battery_pct_from_voltage_mv;
pub use median::median;
pub use obstacle_blocked::obstacle_blocked;
pub use sensor_status::sensor_status;
pub use status_str::status_str;

#[cfg(test)]
mod tests;