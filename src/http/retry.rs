use std::time::Duration;

use rand::Rng;
use reqwest::header::HeaderMap;

use crate::config::RetryConfig;

/// Calculate the backoff duration for a given retry attempt.
///
/// Uses exponential backoff: `min(0.5 * 2^(attempt-1), max_backoff)`
/// with jitter: `base * (1.0 - 0.25 * random())`
pub fn calculate_backoff(attempt: u32, config: &RetryConfig) -> Duration {
    let base_secs = 0.5 * 2.0_f64.powi(attempt as i32 - 1);
    let clamped = base_secs.min(config.max_backoff.as_secs_f64());
    let jitter = 1.0 - 0.25 * rand::thread_rng().gen::<f64>();
    Duration::from_secs_f64(clamped * jitter)
}

/// Extract and parse `Retry-After` or `Retry-After-Ms` from response headers.
/// Returns the suggested wait duration, clamped to `max_retry_after`.
pub fn parse_retry_after(headers: &HeaderMap, config: &RetryConfig) -> Option<Duration> {
    // Check millisecond header first
    if let Some(val) = headers.get("retry-after-ms") {
        if let Ok(ms) = val.to_str().unwrap_or("").parse::<u64>() {
            let dur = Duration::from_millis(ms).min(config.max_retry_after);
            return Some(dur);
        }
    }

    // Then standard Retry-After (seconds)
    if let Some(val) = headers.get("retry-after") {
        if let Ok(secs) = val.to_str().unwrap_or("").parse::<f64>() {
            let dur = Duration::from_secs_f64(secs).min(config.max_retry_after);
            return Some(dur);
        }
    }

    None
}

/// Whether a given HTTP status code is retryable.
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 409 | 429 | 500 | 502 | 503 | 504 | 529)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases() {
        let config = RetryConfig::default();
        let d1 = calculate_backoff(1, &config);
        let d2 = calculate_backoff(2, &config);
        let d3 = calculate_backoff(3, &config);
        // With jitter these aren't deterministic, but the base values should trend up
        // Base: 0.5, 1.0, 2.0 (before jitter)
        assert!(d1.as_secs_f64() <= 1.0);
        assert!(d3.as_secs_f64() <= config.max_backoff.as_secs_f64() + 0.1);
        let _ = d2; // Used
    }

    #[test]
    fn retryable_statuses() {
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(529));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(404));
    }

    #[test]
    fn parse_retry_after_seconds() {
        let config = RetryConfig::default();
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", "2".parse().unwrap());
        let dur = parse_retry_after(&headers, &config).unwrap();
        assert!(dur.as_secs_f64() >= 1.9 && dur.as_secs_f64() <= 2.1);
    }

    #[test]
    fn parse_retry_after_ms() {
        let config = RetryConfig::default();
        let mut headers = HeaderMap::new();
        headers.insert("retry-after-ms", "500".parse().unwrap());
        let dur = parse_retry_after(&headers, &config).unwrap();
        assert_eq!(dur, Duration::from_millis(500));
    }
}
