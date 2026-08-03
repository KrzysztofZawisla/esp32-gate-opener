use esp_idf_svc::http::server::{EspHttpConnection, Request};

use crate::config_storage;
use crate::pure::constant_time_eq;

pub(crate) fn check_auth(request: &Request<&mut EspHttpConnection<'_>>) -> bool {
    let key = config_storage::http_api_key();
    if key.is_empty() {
        return true;
    }
    match request.header("X-Api-Key") {
        Some(provided) => constant_time_eq(provided.as_bytes(), key.as_bytes()),
        None => false,
    }
}
