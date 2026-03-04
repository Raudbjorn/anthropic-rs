# anthropic-rs

Feature-complete, unofficial Rust SDK for the [Anthropic Claude API](https://docs.anthropic.com/en/api).

Covers the full API surface: messages, streaming, tool use, extended thinking, structured output, batches, models, files, web search, code execution, and more. Supports the direct Anthropic API plus AWS Bedrock, Google Vertex AI, and Azure AI Foundry backends behind feature flags. Builds for both native and **WebAssembly** targets.

> **Status:** Pre-release (`0.1.0`). API surface may change before `1.0`.

## Quick Start

```toml
[dependencies]
anthropic-rs = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use anthropic_rs::{Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What is the capital of France?")
        .build();

    let message = client.messages_create(params).await?;
    println!("{}", message.text());
    Ok(())
}
```

Set your API key:

```sh
export ANTHROPIC_API_KEY=sk-ant-...
cargo run --example basic_message
```

## Features

| Feature | Default | Description |
|---|---|---|
| `rustls-tls` | Yes | TLS via rustls (pure Rust) |
| `native-tls` | No | TLS via platform-native library |
| `bedrock` | No | AWS Bedrock Runtime backend (adds `hmac`, `sha2`) |
| `vertex` | No | Google Vertex AI backend |
| `foundry` | No | Azure AI Foundry backend |
| `beta` | No | Beta APIs: files, skills |
| `blocking` | No | Synchronous wrapper (spawns a tokio runtime) |

Enable cloud backends:

```toml
anthropic-rs = { version = "0.1", features = ["bedrock", "vertex"] }
```

## Configuration

### Environment Variables

The SDK reads configuration from environment variables, with programmatic overrides taking precedence.

#### Direct Anthropic API

| Variable | Required | Default | Description |
|---|---|---|---|
| `ANTHROPIC_API_KEY` | Yes* | — | API key (`sk-ant-...`). Used as `x-api-key` header. |
| `ANTHROPIC_AUTH_TOKEN` | No | — | Bearer token. Used as `Authorization: Bearer <token>` header. |
| `ANTHROPIC_BASE_URL` | No | `https://api.anthropic.com` | API base URL. Useful for proxies. |

\* Either `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` must be set (or passed to the builder).

#### AWS Bedrock (`feature = "bedrock"`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `AWS_ACCESS_KEY_ID` | Yes* | — | IAM access key for SigV4 signing. |
| `AWS_SECRET_ACCESS_KEY` | Yes* | — | IAM secret key for SigV4 signing. |
| `AWS_SESSION_TOKEN` | No | — | Session token (for temporary credentials / STS). |
| `AWS_BEARER_TOKEN_BEDROCK` | No | — | Alternative: bearer token auth (instead of SigV4). |
| `AWS_REGION` | Yes | — | AWS region (e.g., `us-east-1`). Falls back to `AWS_DEFAULT_REGION`. |

\* Either `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` or `AWS_BEARER_TOKEN_BEDROCK` must be set.

#### Google Vertex AI (`feature = "vertex"`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `ANTHROPIC_VERTEX_ACCESS_TOKEN` | Yes | — | Google OAuth2 access token (`ya29.a0...`). |
| `CLOUD_ML_REGION` | Yes | — | GCP region (e.g., `us-central1`, `europe-west4`). |
| `ANTHROPIC_VERTEX_PROJECT_ID` | Yes | — | GCP project ID. |

#### Azure AI Foundry (`feature = "foundry"`)

| Variable | Required | Default | Description |
|---|---|---|---|
| `ANTHROPIC_FOUNDRY_API_KEY` | Yes | — | Foundry API key. |
| `ANTHROPIC_FOUNDRY_RESOURCE` | Yes* | — | Azure resource name. Constructs URL: `https://<resource>.services.ai.azure.com`. |
| `ANTHROPIC_FOUNDRY_BASE_URL` | No | — | Full base URL (alternative to resource name). |

\* Either `ANTHROPIC_FOUNDRY_RESOURCE` or `ANTHROPIC_FOUNDRY_BASE_URL` must be set.

### Client Construction

#### Minimal (env vars only)

```rust
let client = Anthropic::from_env()?;
```

Reads `ANTHROPIC_API_KEY` (required), `ANTHROPIC_AUTH_TOKEN`, and `ANTHROPIC_BASE_URL` from the environment.

#### Builder with explicit API key

```rust
let client = Anthropic::builder()
    .api_key("sk-ant-...")
    .build()?;
```

#### Full configuration

```rust
use anthropic_rs::{Anthropic, Timeout, RetryConfig};
use std::time::Duration;

let client = Anthropic::builder()
    .api_key("sk-ant-...")
    .auth_token("bearer-token")        // optional, alongside or instead of api_key
    .base_url("https://my-proxy.com")  // optional, overrides ANTHROPIC_BASE_URL
    .betas(vec![                        // optional, overrides default beta features
        "interleaved-thinking-2025-05-14".into(),
        "code-execution-2025-05-22".into(),
        "my-custom-beta".into(),
    ])
    .timeout(Timeout {
        connect: Duration::from_secs(10),
        request: Duration::from_secs(300),
    })
    .retry_config(RetryConfig {
        max_retries: 3,
        initial_backoff: Duration::from_millis(500),
        max_backoff: Duration::from_secs(8),
        max_retry_after: Duration::from_secs(60),
    })
    .build()?;
```

#### Cloud backends

Each backend has its own builder with `from_env()` for environment-driven configuration:

```rust
// AWS Bedrock
use anthropic_rs::backends::{BedrockBackend, AwsCredentials};

let backend = BedrockBackend::from_env()?;
// or:
let backend = BedrockBackend::builder()
    .credentials(AwsCredentials {
        access_key_id: "AKIA...".into(),
        secret_access_key: "secret".into(),
        session_token: None,
    })
    .region("us-east-1")
    .build()?;

let client = Anthropic::builder().backend(backend).build()?;
```

```rust
// Google Vertex AI
use anthropic_rs::backends::VertexBackend;

let backend = VertexBackend::from_env()?;
// or:
let backend = VertexBackend::builder()
    .access_token("ya29.a0AfH6SM...")
    .region("us-central1")
    .project("my-gcp-project")
    .build()?;

let client = Anthropic::builder().backend(backend).build()?;
```

```rust
// Azure AI Foundry
use anthropic_rs::backends::FoundryBackend;

let backend = FoundryBackend::from_env()?;
// or:
let backend = FoundryBackend::builder()
    .api_key("my-foundry-key")
    .resource("my-resource")  // or .base_url("https://...")
    .build()?;

let client = Anthropic::builder().backend(backend).build()?;
```

### Defaults

| Setting | Default |
|---|---|
| Connect timeout | 30 seconds |
| Request timeout | 600 seconds (10 min, allows for streaming) |
| Max retries | 2 |
| Initial backoff | 500ms |
| Max backoff | 8 seconds |
| Max Retry-After | 60 seconds (clamped) |
| Beta features | `interleaved-thinking-2025-05-14`, `code-execution-2025-05-22` |
| API version header | `2023-06-01` |

### Retry Behavior

Automatic retries with exponential backoff and jitter:

- **Retryable statuses:** 408, 409, 429, 500-599
- **Retryable errors:** IO errors, connection failures
- **Backoff formula:** `min(0.5 * 2^(attempt-1), max_backoff) * (1.0 - 0.25 * random())`
- **Retry-After / Retry-After-Ms** headers are respected (clamped to `max_retry_after`)
- **Idempotency key** is sent on every request and reused across retries

Disable retries:

```rust
let client = Anthropic::builder()
    .retry_config(RetryConfig::none())
    .build()?;
```

## API Reference

### Messages

```rust
// Non-streaming
let message = client.messages_create(params).await?;
println!("{}", message.text());

// Streaming
let mut stream = client.messages_create_stream(params).await?;
while let Some(event) = stream.next().await {
    // Handle StreamEvent variants
}

// Token counting
let count = client.messages_count_tokens(params).await?;
println!("{} tokens", count.input_tokens);
```

### Streaming

```rust
use anthropic_rs::StreamEvent;
use anthropic_rs::streaming::events::ContentBlockDelta;
use tokio_stream::StreamExt;

let mut stream = client.messages_create_stream(params).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text }, ..
        } => print!("{text}"),
        StreamEvent::MessageStop => println!(),
        _ => {}
    }
}
```

### Tool Use

```rust
use anthropic_rs::{Tool, ToolChoice};
use serde_json::json;

let tool = Tool::new("get_weather", json!({
    "type": "object",
    "properties": {
        "location": { "type": "string" }
    },
    "required": ["location"]
})).with_description("Get the current weather.");

let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
    .user("What's the weather in Tokyo?")
    .tool(tool)
    .tool_choice(ToolChoice::auto())
    .build();

let message = client.messages_create(params).await?;
for tool_use in message.tool_uses() {
    println!("{}: {}", tool_use.name, tool_use.input);
}
```

### Extended Thinking

```rust
use anthropic_rs::ThinkingConfig;

let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 16000)
    .user("Solve this step by step: 127 * 349")
    .thinking(ThinkingConfig::Enabled { budget_tokens: 10000 })
    .build();

let message = client.messages_create(params).await?;
println!("Thinking: {}", message.thinking());
println!("Answer: {}", message.text());
```

### Structured Output

```rust
use anthropic_rs::types::output_config::OutputConfig;
use serde_json::json;

let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
    .user("List the 3 largest countries. Return JSON with name and area_km2.")
    .output(OutputConfig::json_schema(json!({
        "type": "object",
        "properties": {
            "countries": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "area_km2": { "type": "number" }
                    },
                    "required": ["name", "area_km2"]
                }
            }
        },
        "required": ["countries"]
    })))
    .build();
```

### Web Search (Server Tool)

```rust
use anthropic_rs::types::web_search::WebSearchTool;

let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 4096)
    .user("What are the latest Rust developments?")
    .tool(WebSearchTool::new())
    .build();
```

### Batches

```rust
use anthropic_rs::batches::{BatchCreateParams, BatchRequest};

// Create
let batch = client.batches_create(params).await?;

// Poll
let batch = client.batches_retrieve(&batch.id).await?;

// List
let page = client.batches_list(Default::default()).await?;

// Stream results
let mut results = client.batches_results_stream(&batch.id).await?;
```

### Models

```rust
let models = client.models_list(Default::default()).await?;
let model = client.models_retrieve("claude-sonnet-4-6-20250514").await?;
```

### Files (Beta)

Requires `features = ["beta"]`.

```rust
let file = client.files_upload(data, "doc.pdf", "user_upload").await?;
let content = client.files_download(&file.id).await?;
let files = client.files_list(Default::default()).await?;
client.files_delete(&file.id).await?;
```

## Error Handling

All methods return `Result<T, AnthropicError>`. HTTP errors carry status, headers, and body:

```rust
use anthropic_rs::{AnthropicError, error::HttpErrorKind};

match client.messages_create(params).await {
    Ok(msg) => println!("{}", msg.text()),
    Err(AnthropicError::Api(details)) => {
        eprintln!("HTTP {}: {}", details.status, details.message);
        match details.kind {
            HttpErrorKind::RateLimited => { /* back off */ }
            HttpErrorKind::Unauthorized => { /* check API key */ }
            _ => {}
        }
    }
    Err(e) => eprintln!("Error: {e}"),
}
```

Error variants:

| Variant | Description |
|---|---|
| `Api(HttpErrorDetails)` | HTTP 4xx/5xx with kind, status, headers, body |
| `Http(reqwest::Error)` | Connection/transport errors |
| `Serialization(serde_json::Error)` | JSON parse failures |
| `InvalidData(String)` | Unexpected response format |
| `Config(String)` | Missing API key, invalid URL, etc. |
| `Sse(String)` | SSE stream parse error |
| `Io(std::io::Error)` | IO errors |

HTTP error kinds: `BadRequest` (400), `Unauthorized` (401), `Billing` (402), `PermissionDenied` (403), `NotFound` (404), `UnprocessableEntity` (422), `RateLimited` (429), `InternalServer` (500-599), `GatewayTimeout` (504), `Overloaded` (529).

## Architecture

The SDK uses a `Backend` trait to abstract over different API providers:

```
Anthropic (client)
  |
  +-- Backend trait (dyn dispatch)
  |     |
  |     +-- AnthropicBackend (default: api.anthropic.com)
  |     +-- BedrockBackend  (AWS SigV4 + event-stream decoding)
  |     +-- VertexBackend   (GCP OAuth2 + URL rewriting)
  |     +-- FoundryBackend  (Azure API key + path prefix)
  |
  +-- reqwest::Client (HTTP transport)
  +-- RetryConfig (exponential backoff)
```

Each backend implements three methods:
1. `prepare_request` — rewrite path segments, body, version headers
2. `authorize_request` — add auth headers / sign the request
3. `stream_transformer` — convert backend-specific streaming format (Bedrock only)

## Examples

Run the examples (requires `ANTHROPIC_API_KEY`):

```sh
cargo run --example basic_message
cargo run --example streaming
cargo run --example tool_use
cargo run --example extended_thinking
cargo run --example structured_output
cargo run --example web_search
cargo run --example batch_processing
cargo run --example file_upload --features beta
cargo run --example bedrock --features bedrock
cargo run --example vertex --features vertex
cargo run --example foundry --features foundry
```

## WebAssembly Support

The SDK compiles for `wasm32-unknown-unknown` out of the box:

```sh
cargo build --target wasm32-unknown-unknown --no-default-features
```

WASM builds use the browser `fetch` API (via reqwest) and JS-based timers for retry backoff. The following adaptations are automatic:

- **No TLS feature needed** — the browser handles TLS
- **Timeouts** — `connect_timeout` and `request_timeout` are ignored (not supported by browser fetch)
- **Sleep** — uses `globalThis.setTimeout` via JS interop (works in browsers, Node.js, Deno, and Cloudflare Workers)
- **Spawning** — uses `wasm_bindgen_futures::spawn_local` instead of `tokio::spawn`
- **Randomness** — `uuid` and `getrandom` use the JS crypto API

Features not available in WASM:
- `blocking` — requires a tokio runtime
- `rustls-tls` / `native-tls` — browser handles TLS

## Supported Models

The `Model` enum provides type-safe constants for all known Claude models. New/unknown models work via `Model::Other("model-id".into())`.

```rust
use anthropic_rs::Model;

Model::ClaudeOpus4_6          // claude-opus-4-6-20250805
Model::ClaudeSonnet4_6        // claude-sonnet-4-6-20250514
Model::ClaudeHaiku4_5         // claude-haiku-4-5-20251001
Model::ClaudeSonnet4_0        // claude-sonnet-4-0-20250514
Model::ClaudeOpus4_0          // claude-opus-4-0-20250514
Model::Claude3_5Sonnet        // claude-3-5-sonnet-20241022
Model::Claude3_5Haiku         // claude-3-5-haiku-20241022
Model::Claude3Opus            // claude-3-opus-20240229
Model::Other("custom".into()) // any model ID string
```

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
