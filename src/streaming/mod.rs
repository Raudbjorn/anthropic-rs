pub mod accumulator;
pub mod events;
pub mod sse;

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::backends::StreamTransformer;
use crate::error::{AnthropicError, Result};
use self::accumulator::MessageAccumulator;
use self::events::RawMessageStreamEvent;
use self::sse::{SseEvent, SseParser};
use crate::types::message::Message;

pub use events::RawMessageStreamEvent as StreamEvent;

pin_project! {
    /// A stream of message events from the streaming API.
    ///
    /// Wraps reqwest's byte stream → optional stream transformer → SSE parser → event deserialization.
    pub struct MessageStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send>>,
        parser: SseParser,
        event_buffer: Vec<SseEvent>,
        transformer: Option<Box<dyn StreamTransformer>>,
        transform_buf: Vec<u8>,
    }
}

impl MessageStream {
    pub(crate) fn new(
        byte_stream: Pin<Box<dyn Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Send>>,
        transformer: Option<Box<dyn StreamTransformer>>,
    ) -> Self {
        Self {
            inner: byte_stream,
            parser: SseParser::new(),
            event_buffer: Vec::new(),
            transformer,
            transform_buf: Vec::new(),
        }
    }

    /// Collect all events into a single `Message` using the accumulator.
    pub async fn collect_message(mut self) -> Result<Message>
    where
        Self: Unpin,
    {
        use tokio_stream::StreamExt;
        let mut acc = MessageAccumulator::new();
        while let Some(event) = StreamExt::next(&mut self).await {
            let event = event?;
            acc.process(&event)?;
        }
        acc.finish()
    }
}

impl Stream for MessageStream {
    type Item = Result<RawMessageStreamEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // Drain buffered events first
        while !this.event_buffer.is_empty() {
            let sse = this.event_buffer.remove(0);
            let event_type = sse.event.as_deref();
            // Skip ping events at the SSE level, they're just keepalives
            if event_type == Some("ping") {
                continue;
            }
            return match deserialize_sse_event(&sse) {
                Ok(Some(event)) => Poll::Ready(Some(Ok(event))),
                Ok(None) => {
                    // Not a recognized event, skip — continue draining
                    continue;
                }
                Err(e) => Poll::Ready(Some(Err(e))),
            };
        }

        // Poll the underlying byte stream
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                // Apply stream transformer if present (e.g., Bedrock event-stream → SSE)
                let sse_bytes = if let Some(ref mut transformer) = this.transformer {
                    this.transform_buf.clear();
                    if let Err(e) = transformer.transform(&bytes, this.transform_buf) {
                        return Poll::Ready(Some(Err(e)));
                    }
                    Bytes::copy_from_slice(this.transform_buf)
                } else {
                    bytes
                };

                let events = this.parser.feed(&sse_bytes);
                *this.event_buffer = events;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(AnthropicError::Http(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn deserialize_sse_event(sse: &SseEvent) -> Result<Option<RawMessageStreamEvent>> {
    let event_type = sse.event.as_deref().unwrap_or("");
    match event_type {
        "message_start" | "content_block_start" | "content_block_delta"
        | "content_block_stop" | "message_delta" | "message_stop" | "ping" | "error" => {
            // Wrap data with type for tagged deserialization
            let mut value: serde_json::Value = serde_json::from_str(&sse.data)
                .map_err(|e| AnthropicError::Sse(format!("invalid JSON in SSE data: {e}")))?;
            if let Some(obj) = value.as_object_mut() {
                obj.insert("type".to_owned(), serde_json::Value::String(event_type.to_owned()));
            }
            let event: RawMessageStreamEvent = serde_json::from_value(value)?;
            Ok(Some(event))
        }
        _ => Ok(None), // Unknown event type, skip
    }
}
