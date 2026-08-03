use crate::config::FAULT_TOPIC;
use crate::state;

use super::publish_raw;

pub fn publish_fault() {
    let payload: &[u8] = if state::fault().is_empty() {
        b"off"
    } else {
        b"on"
    };
    publish_raw(FAULT_TOPIC, payload, true);
}
