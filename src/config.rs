use std::time::Duration;

/// Configuration for HTTP request timeouts.
#[derive(Debug, Clone)]
pub struct Timeout {
    /// Timeout for establishing a connection.
    pub connect: Duration,
    /// Timeout for the entire request (including response body).
    pub request: Duration,
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(30),
            request: Duration::from_secs(600), // 10 min for streaming
        }
    }
}

/// Configuration for automatic retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries (0 = no retries).
    pub max_retries: u32,
    /// Initial backoff duration.
    pub initial_backoff: Duration,
    /// Maximum backoff duration.
    pub max_backoff: Duration,
    /// Maximum value for Retry-After header (clamped).
    pub max_retry_after: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(8),
            max_retry_after: Duration::from_secs(60),
        }
    }
}

impl RetryConfig {
    /// No retries.
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }
}
