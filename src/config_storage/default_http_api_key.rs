pub(crate) fn default_http_api_key() -> &'static str {
    option_env!("HTTP_API_KEY").unwrap_or("")
}
