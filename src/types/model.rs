use serde::{Deserialize, Serialize};
use std::fmt;

/// Known Claude model identifiers with forward-compatible `Other` variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Model {
    // --- Claude 4.6 ---
    ClaudeOpus4_6,
    ClaudeSonnet4_6,

    // --- Claude 4.5 ---
    ClaudeOpus4_5_20251101,
    ClaudeOpus4_5,
    ClaudeSonnet4_5_20250929,
    ClaudeSonnet4_5,
    ClaudeHaiku4_5_20251001,
    ClaudeHaiku4_5,

    // --- Claude 4.1 ---
    ClaudeOpus4_1_20250805,

    // --- Claude 4 ---
    ClaudeSonnet4_20250514,
    ClaudeSonnet4_0,
    Claude4Sonnet20250514,
    ClaudeOpus4_20250514,
    ClaudeOpus4_0,
    Claude4Opus20250514,

    // --- Claude 3.7 (deprecated) ---
    Claude3_7SonnetLatest,
    Claude3_7Sonnet20250219,

    // --- Claude 3.5 (deprecated) ---
    Claude3_5Sonnet20241022,
    Claude3_5SonnetLatest,
    Claude3_5Haiku20241022,
    Claude3_5HaikuLatest,

    // --- Claude 3 (deprecated) ---
    Claude3OpusLatest,
    Claude3Opus20240229,
    Claude3Sonnet20240229,
    Claude3Haiku20240307,

    /// Forward-compatible: any model string not matched above.
    Other(String),
}

impl Model {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ClaudeOpus4_6 => "claude-opus-4-6",
            Self::ClaudeSonnet4_6 => "claude-sonnet-4-6",
            Self::ClaudeOpus4_5_20251101 => "claude-opus-4-5-20251101",
            Self::ClaudeOpus4_5 => "claude-opus-4-5",
            Self::ClaudeSonnet4_5_20250929 => "claude-sonnet-4-5-20250929",
            Self::ClaudeSonnet4_5 => "claude-sonnet-4-5",
            Self::ClaudeHaiku4_5_20251001 => "claude-haiku-4-5-20251001",
            Self::ClaudeHaiku4_5 => "claude-haiku-4-5",
            Self::ClaudeOpus4_1_20250805 => "claude-opus-4-1-20250805",
            Self::ClaudeSonnet4_20250514 => "claude-sonnet-4-20250514",
            Self::ClaudeSonnet4_0 => "claude-sonnet-4-0",
            Self::Claude4Sonnet20250514 => "claude-4-sonnet-20250514",
            Self::ClaudeOpus4_20250514 => "claude-opus-4-20250514",
            Self::ClaudeOpus4_0 => "claude-opus-4-0",
            Self::Claude4Opus20250514 => "claude-4-opus-20250514",
            Self::Claude3_7SonnetLatest => "claude-3-7-sonnet-latest",
            Self::Claude3_7Sonnet20250219 => "claude-3-7-sonnet-20250219",
            Self::Claude3_5Sonnet20241022 => "claude-3-5-sonnet-20241022",
            Self::Claude3_5SonnetLatest => "claude-3-5-sonnet-latest",
            Self::Claude3_5Haiku20241022 => "claude-3-5-haiku-20241022",
            Self::Claude3_5HaikuLatest => "claude-3-5-haiku-latest",
            Self::Claude3OpusLatest => "claude-3-opus-latest",
            Self::Claude3Opus20240229 => "claude-3-opus-20240229",
            Self::Claude3Sonnet20240229 => "claude-3-sonnet-20240229",
            Self::Claude3Haiku20240307 => "claude-3-haiku-20240307",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for Model {
    fn from(s: &str) -> Self {
        match s {
            "claude-opus-4-6" => Self::ClaudeOpus4_6,
            "claude-sonnet-4-6" => Self::ClaudeSonnet4_6,
            "claude-opus-4-5-20251101" => Self::ClaudeOpus4_5_20251101,
            "claude-opus-4-5" => Self::ClaudeOpus4_5,
            "claude-sonnet-4-5-20250929" => Self::ClaudeSonnet4_5_20250929,
            "claude-sonnet-4-5" => Self::ClaudeSonnet4_5,
            "claude-haiku-4-5-20251001" => Self::ClaudeHaiku4_5_20251001,
            "claude-haiku-4-5" => Self::ClaudeHaiku4_5,
            "claude-opus-4-1-20250805" => Self::ClaudeOpus4_1_20250805,
            "claude-sonnet-4-20250514" => Self::ClaudeSonnet4_20250514,
            "claude-sonnet-4-0" => Self::ClaudeSonnet4_0,
            "claude-4-sonnet-20250514" => Self::Claude4Sonnet20250514,
            "claude-opus-4-20250514" => Self::ClaudeOpus4_20250514,
            "claude-opus-4-0" => Self::ClaudeOpus4_0,
            "claude-4-opus-20250514" => Self::Claude4Opus20250514,
            "claude-3-7-sonnet-latest" => Self::Claude3_7SonnetLatest,
            "claude-3-7-sonnet-20250219" => Self::Claude3_7Sonnet20250219,
            "claude-3-5-sonnet-20241022" => Self::Claude3_5Sonnet20241022,
            "claude-3-5-sonnet-latest" => Self::Claude3_5SonnetLatest,
            "claude-3-5-haiku-20241022" => Self::Claude3_5Haiku20241022,
            "claude-3-5-haiku-latest" => Self::Claude3_5HaikuLatest,
            "claude-3-opus-latest" => Self::Claude3OpusLatest,
            "claude-3-opus-20240229" => Self::Claude3Opus20240229,
            "claude-3-sonnet-20240229" => Self::Claude3Sonnet20240229,
            "claude-3-haiku-20240307" => Self::Claude3Haiku20240307,
            other => Self::Other(other.to_owned()),
        }
    }
}

impl From<String> for Model {
    fn from(s: String) -> Self {
        Model::from(s.as_str())
    }
}

impl Serialize for Model {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Model::from(s))
    }
}
