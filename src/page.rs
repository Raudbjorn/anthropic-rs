use serde::Deserialize;

/// A paginated list response.
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

impl<T> Page<T> {
    /// Whether there are more results to fetch.
    pub fn has_more(&self) -> bool {
        self.has_more
    }

    /// The ID to use as `after_id` for the next page.
    pub fn next_cursor(&self) -> Option<&str> {
        self.last_id.as_deref()
    }
}
