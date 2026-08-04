use anyhow::Result;

/// Parses a URL-encoded HTTP query string into ordered key/value pairs.
///
/// Duplicate keys are preserved in order; consumers apply the last occurrence.
/// Malformed percent-encoding or non-UTF-8 input returns an error.
pub fn parse_config_query(query: &str) -> Result<Vec<(String, String)>> {
    Ok(serde_urlencoded::from_str(query)?)
}
