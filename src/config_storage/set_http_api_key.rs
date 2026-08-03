use log::warn;

use super::{HTTP_API_KEY, KEY_HTTP_API_KEY, NVS};

pub fn set_http_api_key(value: &str) -> bool {
    let mut guard = NVS.lock().unwrap();
    let Some(nvs) = guard.as_mut() else {
        return false;
    };
    match nvs.set_str(KEY_HTTP_API_KEY, value) {
        Ok(()) => {
            *HTTP_API_KEY.lock().unwrap() = value.to_string();
            true
        }
        Err(error) => {
            warn!("Failed to persist api_key: {error}");
            false
        }
    }
}
