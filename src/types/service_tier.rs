use serde::{Deserialize, Serialize};

/// Service tier for request prioritization (used in MessageCreateParams).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    /// Automatically select tier.
    Auto,
    /// Only use standard tier.
    StandardOnly,
}

/// Service tier reported in response usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageServiceTier {
    /// Standard tier.
    Standard,
    /// Priority tier (higher QoS).
    Priority,
    /// Batch tier (lower priority, lower cost).
    Batch,
}
