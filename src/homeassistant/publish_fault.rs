use crate::config::FAULT_TOPIC;
use crate::state;

use super::publish_raw;

pub fn publish_fault() {
    let payload: &[u8] = if state::fault() != 0 { b"on" } else { b"off" };
    publish_raw(FAULT_TOPIC, payload, true);
}