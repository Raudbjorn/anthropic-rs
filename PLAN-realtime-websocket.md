# Realtime API WebSocket Support - Implementation Plan

## Overview

Add OpenAI Realtime API support via WebSocket to `anthropic-rs`. This is a
server-to-server WebSocket integration for real-time speech-to-speech
conversations with models like `gpt-realtime`. Authentication uses a standard
API key in the WebSocket handshake headers.

## Architecture

The Realtime API is fundamentally different from the existing REST/SSE-based
Messages API. It uses a **persistent, bidirectional WebSocket** connection with
JSON-serialized events flowing in both directions. This requires new
abstractions rather than extending the existing `Backend` trait.

```text
anthropic-rs/src/
  realtime/
    mod.rs              # Module root, public re-exports
    types.rs            # Shared types (Session, ConversationItem, Voice, etc.)
    client_events.rs    # All client-sent event types (11 variants)
    server_events.rs    # All server-sent event types (28 variants)
    client.rs           # RealtimeClient + RealtimeConfig (native only)
    session.rs          # Session state tracking
    conversation.rs     # Client-side conversation cache
    audio.rs            # Base64 audio encoding/decoding, PCM16 helpers
    error.rs            # Realtime-specific errors (close codes, event errors)
    platform.rs         # WebSocket platform abstraction (native only)
```

## Phases

### Phase 1: Core Types & Events

Define all the event types as Rust structs with serde. This is the foundation.

**Files:** `realtime/events/`

**Client events** (sent by us):
- `session.update` - Configure session (model, voice, VAD, tools, etc.)
- `input_audio_buffer.append` - Stream audio chunks (base64)
- `input_audio_buffer.commit` - Commit buffered audio as user turn
- `input_audio_buffer.clear` - Clear the audio buffer
- `conversation.item.create` - Add text/audio/image message to conversation
- `conversation.item.truncate` - Truncate model audio after interruption
- `conversation.item.delete` - Remove an item
- `response.create` - Trigger model response
- `response.cancel` - Cancel in-progress response

**Server events** (received from API):
- `session.created` / `session.updated`
- `input_audio_buffer.committed` / `.cleared`
- `input_audio_buffer.speech_started` / `.speech_stopped`
- `conversation.created`
- `conversation.item.created` / `.deleted`
- `conversation.item.input_audio_transcription.completed` / `.failed`
- `response.created` / `.done` / `.cancelled`
- `response.output_item.added` / `.done`
- `response.content_part.added` / `.done`
- `response.text.delta` / `.done`
- `response.audio.delta` / `.done`
- `response.audio_transcript.delta` / `.done`
- `response.function_call_arguments.delta` / `.done`
- `rate_limits.updated`
- `error`

**Common types:**
- `RealtimeSession` - session config object
- `ConversationItem` - message / function_call / function_call_output
- `ContentPart` - text / audio / image content
- `AudioConfig` - format, sample rate, VAD settings
- `TurnDetection` - `server_vad` / `semantic_vad` / `null` (disabled)
- `RealtimeTool` - function tool definition
- `ToolChoice` - auto / none / required / specific function
- `Voice` - alloy, ash, ballad, coral, echo, sage, shimmer, verse, marin, cedar
- `AudioFormat` - pcm16, g711_ulaw, g711_alaw
- `RealtimeModel` - model identifiers

### Phase 2: WebSocket Platform Abstraction

WebSocket support differs between native and WASM. Abstract this like existing
`platform.rs` does for sleep/spawn.

**File:** `realtime/platform.rs`

**Native (tokio):**
- Use `tokio-tungstenite` for async WebSocket
- TLS via `rustls` (or `native-tls` matching feature flags)
- Auth header in handshake request

**WASM:**
- Use `web-sys` WebSocket API
- Wrap in async Stream/Sink interface
- Auth via protocol header or query param (browser WS has limited header support)

**Trait:**
```rust
pub(crate) trait WebSocketConnection: Send {
    async fn send_text(&mut self, msg: String) -> Result<()>;
    async fn recv(&mut self) -> Option<Result<WsMessage>>;
    async fn close(&mut self) -> Result<()>;
}

pub(crate) enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Close { code: u16, reason: String },
}
```

### Phase 3: RealtimeClient

The main client that manages the WebSocket connection.

**File:** `realtime/client.rs`

```rust
pub struct RealtimeClient {
    ws: Box<dyn WebSocketConnection>,
    session: RealtimeSession,
    conversation: ConversationCache,
    config: RealtimeConfig,
}

impl RealtimeClient {
    /// Connect to the Realtime API.
    pub async fn connect(config: RealtimeConfig) -> Result<Self>;

    /// Send a client event.
    pub async fn send(&mut self, event: ClientEvent) -> Result<()>;

    /// Receive the next server event.
    pub async fn recv(&mut self) -> Option<Result<ServerEvent>>;

    /// Returns a Stream of server events.
    pub fn events(&mut self) -> impl Stream<Item = Result<ServerEvent>>;

    /// Update session configuration.
    pub async fn update_session(&mut self, session: SessionUpdate) -> Result<()>;

    /// Send a user text message and trigger a response.
    pub async fn send_text(&mut self, text: &str) -> Result<()>;

    /// Append audio chunk to input buffer (base64).
    pub async fn append_audio(&mut self, audio: &[u8]) -> Result<()>;

    /// Commit audio buffer and create response.
    pub async fn commit_audio(&mut self) -> Result<()>;

    /// Create a conversation item.
    pub async fn create_item(&mut self, item: ConversationItem) -> Result<()>;

    /// Trigger a model response.
    pub async fn create_response(&mut self, params: Option<ResponseCreate>) -> Result<()>;

    /// Cancel in-progress response.
    pub async fn cancel_response(&mut self) -> Result<()>;

    /// Close the connection.
    pub async fn close(self) -> Result<()>;
}
```

### Phase 4: Session & Conversation State

Track session and conversation state client-side.

**File:** `realtime/session.rs`, `realtime/conversation.rs`

- Parse `session.created` / `session.updated` to maintain local session state
- Cache conversation items as they arrive via server events
- Handle item truncation for audio interruption
- Track response lifecycle (created -> in_progress -> done/cancelled)

### Phase 5: Audio Utilities

Helpers for audio data handling.

**File:** `realtime/audio.rs`

```rust
/// Encode raw PCM16 audio to base64 for input_audio_buffer.append.
pub fn encode_audio_base64(pcm16: &[i16]) -> String;

/// Decode base64 audio from response.audio.delta to raw bytes.
pub fn decode_audio_base64(b64: &str) -> Result<Vec<u8>>;

/// Convert f32 audio samples to PCM16 i16 samples.
pub fn float_to_pcm16(samples: &[f32]) -> Vec<i16>;

/// Convert PCM16 i16 samples to f32 audio.
pub fn pcm16_to_float(samples: &[i16]) -> Vec<f32>;
```

### Phase 6: Function Calling

Support for tool registration and automatic execution.

**Part of:** `realtime/client.rs`

```rust
impl RealtimeClient {
    /// Register a tool with an async handler.
    pub fn add_tool<F, Fut>(
        &mut self,
        definition: RealtimeTool,
        handler: F,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static;

    /// Process a function_call item: run handler, send output, trigger response.
    async fn handle_function_call(&mut self, item: &ConversationItem) -> Result<()>;
}
```

### Phase 7: Error Handling

Realtime-specific errors integrated with existing `AnthropicError`.

**File:** `realtime/error.rs`

```rust
pub enum RealtimeError {
    /// WebSocket connection failed.
    ConnectionFailed(String),
    /// WebSocket closed unexpectedly.
    ConnectionClosed { code: u16, reason: String },
    /// Server sent an error event.
    ServerError { error_type: String, message: String, event_id: Option<String> },
    /// Invalid event received.
    InvalidEvent(String),
    /// Audio encoding/decoding error.
    AudioError(String),
    /// Session not initialized.
    NotConnected,
}
```

Add `Realtime(RealtimeError)` variant to `AnthropicError`.

### Phase 8: Integration with Existing Client

Wire into `Anthropic` client and feature flags.

**Cargo.toml additions:**
```toml
# Native WebSocket
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-native-roots"], optional = true }

# WASM WebSocket (already have web-sys, just need WebSocket feature)
# web-sys = { features = ["WebSocket", "MessageEvent", "CloseEvent", "ErrorEvent"] }

[features]
realtime = ["dep:tokio-tungstenite"]  # native only initially
```

**Client integration:**
```rust
impl Anthropic {
    /// Connect to the OpenAI Realtime API via WebSocket.
    #[cfg(feature = "realtime")]
    pub async fn realtime_connect(
        &self,
        config: RealtimeConfig,
    ) -> Result<RealtimeClient>;
}
```

### Phase 9: Examples

**File:** `examples/realtime_text.rs`
- Connect, send text, receive text response

**File:** `examples/realtime_audio.rs`
- Connect, stream audio input from file, play audio output

**File:** `examples/realtime_function_calling.rs`
- Register tools, handle function calls

### Phase 10: Tests

- Unit tests for all event serde (roundtrip serialization)
- Unit tests for audio encoding/decoding
- Unit tests for conversation state management
- Integration test with mock WebSocket server (using `tokio-tungstenite`)
- Test session lifecycle events
- Test function calling flow
- Test interruption / truncation

## Dependencies to Add

| Crate | Purpose | Feature-gated |
|---|---|---|
| `tokio-tungstenite` | Native async WebSocket | `realtime` |
| `web-sys` (extra features) | WASM WebSocket | `realtime` + wasm32 |

## Feature Flag

```toml
realtime = ["dep:tokio-tungstenite"]
```

Optional WASM support can follow later since the primary use case is
server-to-server (as stated in OpenAI docs).

## Key Design Decisions

1. **Separate from Backend trait** - The Realtime API is bidirectional WebSocket,
   not request-response HTTP. Trying to fit it into `Backend` would be forced.

2. **Event-driven, not request-response** - The client exposes a `Stream` of
   server events rather than async methods that return responses directly.

3. **Optional conversation cache** - Client-side state tracking mirrors
   OpenAI's reference implementation but is opt-in for users who want raw events.

4. **Audio as `&[u8]` / `Vec<u8>`** - Keep audio format agnostic. The base64
   encoding/decoding happens at the transport layer. Users work with raw bytes.

5. **Native-first** - Server-to-server is the primary use case per OpenAI docs.
   WASM support can be added later without breaking changes.

6. **Typed events** - All client and server events are fully typed Rust enums
   with serde, not `serde_json::Value`. Matches the crate's existing style.

## Implementation Order

1. Types & events (Phase 1) - can be done independently
2. Error types (Phase 7) - needed by everything else
3. Platform abstraction (Phase 2) - needed by client
4. Audio utilities (Phase 5) - standalone, testable
5. RealtimeClient (Phase 3) - core connection logic
6. Session & conversation (Phase 4) - state management
7. Function calling (Phase 6) - builds on client + types
8. Client integration (Phase 8) - wire it all together
9. Examples (Phase 9) - demonstrate usage
10. Tests (Phase 10) - throughout, but comprehensive pass at end
