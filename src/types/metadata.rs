use serde::{Deserialize, Serialize};

/// Request metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    /// Opaque user ID for abuse detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// User location for server tools (e.g., web search regionalization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    /// Always `"approximate"`.
    #[serde(rename = "type", default = "UserLocation::default_type")]
    pub location_type: String,
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// IANA timezone (e.g. "America/New_York").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// City name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Region/state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

impl UserLocation {
    fn default_type() -> String {
        "approximate".to_owned()
    }

    pub fn new() -> Self {
        Self {
            location_type: "approximate".to_owned(),
            country: None,
            timezone: None,
            city: None,
            region: None,
        }
    }

    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn with_timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = Some(tz.into());
        self
    }

    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
}

impl Default for UserLocation {
    fn default() -> Self {
        Self::new()
    }
}
