//! Shared types for the OpenAI Realtime API.
//!
//! These types are used across both client and server events to represent
//! sessions, conversation items, audio configuration, and tools.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ── Audio & Voice ────────────────────────────────────────────────────

/// Audio encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    /// 16-bit PCM at 24kHz, little-endian.
    Pcm16,
    /// G.711 mu-law at 8kHz.
    #[serde(rename = "g711_ulaw")]
    G711Ulaw,
    /// G.711 A-law at 8kHz.
    #[serde(rename = "g711_alaw")]
    G711Alaw,
}

/// Voice used for audio output.
///
/// Known voices are enumerated; unknown strings are preserved via `Other`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Voice {
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Sage,
    Shimmer,
    Verse,
    Marin,
    Cedar,
    Other(String),
}

impl Serialize for Voice {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::Alloy => "alloy",
            Self::Ash => "ash",
            Self::Ballad => "ballad",
            Self::Coral => "coral",
            Self::Echo => "echo",
            Self::Sage => "sage",
            Self::Shimmer => "shimmer",
            Self::Verse => "verse",
            Self::Marin => "marin",
            Self::Cedar => "cedar",
            Self::Other(s) => s.as_str(),
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for Voice {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "alloy" => Self::Alloy,
            "ash" => Self::Ash,
            "ballad" => Self::Ballad,
            "coral" => Self::Coral,
            "echo" => Self::Echo,
            "sage" => Self::Sage,
            "shimmer" => Self::Shimmer,
            "verse" => Self::Verse,
            "marin" => Self::Marin,
            "cedar" => Self::Cedar,
            _ => Self::Other(s),
        })
    }
}

// ── Modality ─────────────────────────────────────────────────────────

/// Output modality for a session or response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Audio,
}

// ── Realtime Models ──────────────────────────────────────────────────

/// Model identifiers for the Realtime API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealtimeModel {
    GptRealtime,
    Gpt4oRealtimePreview,
    Gpt4oMiniRealtimePreview,
    Other(String),
}

impl RealtimeModel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::GptRealtime => "gpt-realtime",
            Self::Gpt4oRealtimePreview => "gpt-4o-realtime-preview",
            Self::Gpt4oMiniRealtimePreview => "gpt-4o-mini-realtime-preview",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl Serialize for RealtimeModel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RealtimeModel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "gpt-realtime" => Self::GptRealtime,
            s if s.starts_with("gpt-4o-realtime-preview") => Self::Gpt4oRealtimePreview,
            s if s.starts_with("gpt-4o-mini-realtime-preview") => Self::Gpt4oMiniRealtimePreview,
            _ => Self::Other(s),
        })
    }
}

impl std::fmt::Display for RealtimeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Turn Detection ───────────────────────────────────────────────────

/// Voice activity detection / turn detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TurnDetection {
    /// Server-side voice activity detection.
    #[serde(rename = "server_vad")]
    ServerVad {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threshold: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix_padding_ms: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        silence_duration_ms: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        create_response: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interrupt_response: Option<bool>,
    },
    /// Semantic turn detection.
    #[serde(rename = "semantic_vad")]
    SemanticVad {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eagerness: Option<Eagerness>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        create_response: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interrupt_response: Option<bool>,
    },
}

/// Eagerness level for semantic VAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eagerness {
    Low,
    Medium,
    High,
    Auto,
}

// ── Noise Reduction ──────────────────────────────────────────────────

/// Input audio noise reduction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioNoiseReduction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub noise_type: Option<NoiseReductionType>,
}

/// Noise reduction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseReductionType {
    NearField,
    FarField,
}

// ── Transcription ────────────────────────────────────────────────────

/// Input audio transcription configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioTranscription {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

// ── Tools ────────────────────────────────────────────────────────────

/// A function tool definition for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTool {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

impl RealtimeTool {
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            tool_type: Some("function".into()),
            name: Some(name.into()),
            description: Some(description.into()),
            parameters: Some(parameters),
        }
    }
}

// ── Max Output Tokens ────────────────────────────────────────────────

/// Max output tokens: either a numeric limit or `"inf"` for unlimited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaxOutputTokens {
    Limit(u64),
    Inf,
}

impl Serialize for MaxOutputTokens {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Limit(n) => serializer.serialize_u64(*n),
            Self::Inf => serializer.serialize_str("inf"),
        }
    }
}

impl<'de> Deserialize<'de> for MaxOutputTokens {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::Number(n) => {
                let n = n.as_u64().ok_or_else(|| {
                    serde::de::Error::custom("expected positive integer for max_output_tokens")
                })?;
                Ok(Self::Limit(n))
            }
            serde_json::Value::String(s) if s == "inf" => Ok(Self::Inf),
            _ => Err(serde::de::Error::custom(
                "expected integer or \"inf\" for max_output_tokens",
            )),
        }
    }
}

// ── Tracing ──────────────────────────────────────────────────────────

/// Tracing configuration: `"auto"` or a structured config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tracing {
    Auto(String),
    Config(TracingConfig),
}

impl Tracing {
    pub fn auto() -> Self {
        Self::Auto("auto".into())
    }
}

/// Structured tracing configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TracingConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
}

// ── Session ──────────────────────────────────────────────────────────

/// Realtime session configuration.
///
/// Used in `session.update` (client) and `session.created`/`session.updated` (server).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<RealtimeModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<AudioFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<InputAudioTranscription>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: Option<InputAudioNoiseReduction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<MaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing: Option<Tracing>,
}

// ── Conversation Items ───────────────────────────────────────────────

/// The type of a conversation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Message,
    FunctionCall,
    FunctionCallOutput,
}

/// The role of a message item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
}

/// Status of a conversation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Completed,
    Incomplete,
    InProgress,
}

/// Content type within a conversation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    InputText,
    InputAudio,
    InputImage,
    ItemReference,
    Text,
    Audio,
    /// Output text (used in non-beta assistant content).
    OutputText,
    /// Output audio (used in non-beta assistant content).
    OutputAudio,
}

/// A content part within a conversation item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub content_type: Option<ContentType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    /// For `input_image` content: data URI of the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// For `input_image` content: detail level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// For `item_reference` content: the referenced item ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl ContentPart {
    pub fn input_text(text: impl Into<String>) -> Self {
        Self {
            content_type: Some(ContentType::InputText),
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub fn input_audio(audio_base64: impl Into<String>) -> Self {
        Self {
            content_type: Some(ContentType::InputAudio),
            audio: Some(audio_base64.into()),
            ..Default::default()
        }
    }

    pub fn input_image(image_url: impl Into<String>) -> Self {
        Self {
            content_type: Some(ContentType::InputImage),
            image_url: Some(image_url.into()),
            ..Default::default()
        }
    }

    pub fn item_reference(id: impl Into<String>) -> Self {
        Self {
            content_type: Some(ContentType::ItemReference),
            id: Some(id.into()),
            ..Default::default()
        }
    }
}

/// A conversation item (message, function call, or function call output).
///
/// Uses an all-optional flat model to accommodate the different item types
/// in a single struct, matching the OpenAI API's approach.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub item_type: Option<ItemType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentPart>>,

    // Function call fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,

    // Function call output field
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl ConversationItem {
    /// Create a user text message.
    pub fn user_message(text: impl Into<String>) -> Self {
        Self {
            item_type: Some(ItemType::Message),
            role: Some(Role::User),
            content: Some(vec![ContentPart::input_text(text)]),
            ..Default::default()
        }
    }

    /// Create a user audio message.
    pub fn user_audio(audio_base64: impl Into<String>) -> Self {
        Self {
            item_type: Some(ItemType::Message),
            role: Some(Role::User),
            content: Some(vec![ContentPart::input_audio(audio_base64)]),
            ..Default::default()
        }
    }

    /// Create a system message.
    pub fn system_message(text: impl Into<String>) -> Self {
        Self {
            item_type: Some(ItemType::Message),
            role: Some(Role::System),
            content: Some(vec![ContentPart::input_text(text)]),
            ..Default::default()
        }
    }

    /// Create a function call output item.
    pub fn function_call_output(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            item_type: Some(ItemType::FunctionCallOutput),
            call_id: Some(call_id.into()),
            output: Some(output.into()),
            ..Default::default()
        }
    }
}

// ── Response ─────────────────────────────────────────────────────────

/// Status of a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Completed,
    Cancelled,
    Failed,
    Incomplete,
    InProgress,
}

/// Reason for a response status change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusReason {
    TurnDetected,
    ClientCancelled,
    MaxOutputTokens,
    ContentFilter,
}

/// Detailed status information for a response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseStatusDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub status_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<StatusReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// Error details within a response status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// A Realtime API response object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealtimeResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: Option<ResponseStatusDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<ConversationItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<MaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<RealtimeUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Usage statistics for a Realtime response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealtimeUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<InputTokenDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_token_details: Option<OutputTokenDetails>,
}

/// Breakdown of input tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputTokenDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u64>,
}

/// Breakdown of output tokens.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputTokenDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u64>,
}

// ── Rate Limits ──────────────────────────────────────────────────────

/// A rate limit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_seconds: Option<f64>,
}

// ── Conversation ─────────────────────────────────────────────────────

/// A conversation object returned in `conversation.created`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conversation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
}

// ── Error ────────────────────────────────────────────────────────────

/// Error object in server `error` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

// ── Transcription Error ──────────────────────────────────────────────

/// Error object for transcription failures.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptionError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

// ── Response Create Params ───────────────────────────────────────────

/// Parameters for `response.create` client events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseCreateParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Vec<ConversationItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<MaxOutputTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeTool>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
}
