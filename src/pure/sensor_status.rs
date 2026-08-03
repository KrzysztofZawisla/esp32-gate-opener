use super::{ST_CLOSED, ST_ERROR, ST_OPEN, ST_STOPPED};

pub fn sensor_status(open_active: bool, closed_active: bool) -> u8 {
    match (open_active, closed_active) {
        (true, true) => ST_ERROR,
        (true, false) => ST_OPEN,
        (false, true) => ST_CLOSED,
        (false, false) => ST_STOPPED,
    }
}
