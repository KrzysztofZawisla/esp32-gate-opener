use super::{
    STATUS_CLOSED, STATUS_CLOSING, STATUS_ERROR, STATUS_OPEN, STATUS_OPENING, STATUS_STOPPED,
    ST_CLOSED, ST_CLOSING, ST_ERROR, ST_OPEN, ST_OPENING,
};

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