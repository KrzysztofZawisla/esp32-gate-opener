use super::HTTP_API_KEY;

pub fn http_api_key() -> String {
    HTTP_API_KEY.lock().unwrap().clone()
}