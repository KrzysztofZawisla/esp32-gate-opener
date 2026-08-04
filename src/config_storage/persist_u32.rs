use core::sync::atomic::{AtomicU32, Ordering};
use log::warn;

use super::NVS;

pub(crate) fn persist_u32(key: &str, value: u32, slot: &AtomicU32) -> bool {
    let guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_ref() else {
        return false;
    };
    match nvs.set_u32(key, value) {
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
