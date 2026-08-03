use core::sync::atomic::{AtomicU8, Ordering};
use log::warn;

use super::NVS;

pub(crate) fn persist_u8(key: &str, value: u8, slot: &AtomicU8) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u8(key, value) {
        Ok(()) => {
            slot.store(value, Ordering::Relaxed);
            true
        }
        Err(error) => {
            warn!("Failed to persist {key}: {error}");
            false
        }
    }
}
