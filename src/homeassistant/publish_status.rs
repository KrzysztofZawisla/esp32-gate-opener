use crate::config::STATUS_TOPIC;
use crate::state;

use super::publish_raw;

pub fn publish_status() {
    publish_raw(STATUS_TOPIC, state::status().as_bytes(), true);
}
