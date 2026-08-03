use crate::config::OBSTACLE_TOPIC;
use crate::state;

use super::publish_raw;

pub fn publish_obstacle() {
    let payload: &[u8] = if state::obstacle() { b"on" } else { b"off" };
    publish_raw(OBSTACLE_TOPIC, payload, true);
}