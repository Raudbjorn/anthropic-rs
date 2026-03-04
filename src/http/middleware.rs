use reqwest::header::{HeaderMap, HeaderValue};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const ANTHROPIC_BETA: &str = "interleaved-thinking-2025-05-14,code-execution-2025-05-22";

/// Build standard headers for all Anthropic API requests.
pub fn build_headers(api_key: &str, idempotency_key: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        "x-api-key",
        HeaderValue::from_str(api_key).expect("invalid API key characters"),
    );
    headers.insert(
        "anthropic-version",
        HeaderValue::from_static(ANTHROPIC_VERSION),
    );
    headers.insert(
        "anthropic-beta",
        HeaderValue::from_static(ANTHROPIC_BETA),
    );
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    if let Some(key) = idempotency_key {
        if let Ok(val) = HeaderValue::from_str(key) {
            headers.insert("idempotency-key", val);
        }
    }

    headers
}

/// Add retry count header.
pub fn add_retry_headers(headers: &mut HeaderMap, retry_count: u32) {
    if let Ok(val) = HeaderValue::from_str(&retry_count.to_string()) {
        headers.insert("x-stainless-retry-count", val);
    }
}
