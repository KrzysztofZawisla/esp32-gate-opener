use crate::config_storage;

pub(crate) fn grace_ms() -> u64 {
    config_storage::grace_ms() as u64
}
