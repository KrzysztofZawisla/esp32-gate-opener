use crate::config_storage;

pub(crate) fn motion_timeout_ms() -> u64 {
    config_storage::motion_timeout_s() as u64 * 1000
}
