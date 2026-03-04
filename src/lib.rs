//! # anthropic-rs
//!
//! Feature-complete Rust SDK for the Anthropic Claude API.
//!
//! ## Quick Start
//!
//! ```no_run
//! use anthropic_rs::{Anthropic, MessageCreateParams, Model, MessageParam};
//!
//! # async fn example() -> anthropic_rs::Result<()> {
//! let client = Anthropic::from_env()?;
//!
//! let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
//!     .user("Hello, Claude!")
//!     .build();
//!
//! let message = client.messages_create(params).await?;
//! println!("{}", message.text());
//! # Ok(())
//! # }
//! ```
//!
//! ## Cloud Backends
//!
//! Use with AWS Bedrock, Google Vertex AI, or Azure AI Foundry:
//!
//! ```no_run
//! # #[cfg(feature = "bedrock")]
//! # fn example() -> anthropic_rs::Result<()> {
//! use anthropic_rs::{Anthropic, backends::BedrockBackend};
//!
//! let backend = BedrockBackend::from_env()?;
//! let client = Anthropic::builder().backend(backend).build()?;
//! # Ok(())
//! # }
//! ```

pub mod backends;
pub mod batches;
#[cfg(feature = "beta")]
pub mod beta;
#[cfg(all(feature = "blocking", not(target_arch = "wasm32")))]
pub mod blocking;
pub mod client;
pub mod config;
pub mod error;
pub mod http;
pub mod messages;
pub mod models_api;
pub mod page;
pub(crate) mod platform;
pub mod streaming;
pub mod types;

// Re-export the most commonly used items at crate root.
pub use client::{Anthropic, AnthropicBuilder};
pub use config::{RetryConfig, Timeout};
pub use error::{AnthropicError, Result};
pub use messages::{MessageCreateParams, MessageCreateParamsBuilder, SystemPrompt};
pub use page::Page;
pub use streaming::{MessageStream, StreamEvent};
pub use types::*;
