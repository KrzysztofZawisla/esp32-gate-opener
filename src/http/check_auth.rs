use esp_idf_svc::http::server::{EspHttpConnection, Request};

use crate::config_storage;

pub(crate) fn check_auth(request: &Request<&mut EspHttpConnection<'_>>) -> bool {
    let key = config_storage::http_api_key();
    if key.is_empty() {
        return true;
    }
    request.header("X-Api-Key") == Some(key.as_str())
}
