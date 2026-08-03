use core::sync::atomic::{AtomicU16, Ordering};
use log::warn;

use super::NVS;

pub(crate) fn persist_u16(key: &str, value: u16, slot: &AtomicU16) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u16(key, value) {
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
