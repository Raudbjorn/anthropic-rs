use crate::error::{AnthropicError, Result};
use crate::streaming::events::{ContentBlockDelta, RawMessageStreamEvent};
use crate::types::content_block::ContentBlock;
use crate::types::message::Message;
use crate::types::text::TextBlock;
use crate::types::thinking::ThinkingBlock;
use crate::types::tool_use::ToolUseBlock;

/// Accumulates streaming events into a complete `Message`.
pub struct MessageAccumulator {
    message: Option<Message>,
    content_blocks: Vec<AccumulatingBlock>,
}

enum AccumulatingBlock {
    Text { text: String, citations: Vec<crate::types::citation::TextCitation> },
    Thinking { thinking: String, signature: String },
    ToolUse { id: String, name: String, json_buf: String },
    Other(ContentBlock),
}

impl MessageAccumulator {
    pub fn new() -> Self {
        Self {
            message: None,
            content_blocks: Vec::new(),
        }
    }

    /// Process a single stream event.
    pub fn process(&mut self, event: &RawMessageStreamEvent) -> Result<()> {
        match event {
            RawMessageStreamEvent::MessageStart { message } => {
                self.message = Some(message.clone());
                self.content_blocks.clear();
            }
            RawMessageStreamEvent::ContentBlockStart { content_block, .. } => {
                let block = match content_block {
                    ContentBlock::Text(tb) => AccumulatingBlock::Text {
                        text: tb.text.clone(),
                        citations: tb.citations.clone().unwrap_or_default(),
                    },
                    ContentBlock::Thinking(tb) => AccumulatingBlock::Thinking {
                        thinking: tb.thinking.clone(),
                        signature: tb.signature.clone(),
                    },
                    ContentBlock::ToolUse(tu) => AccumulatingBlock::ToolUse {
                        id: tu.id.clone(),
                        name: tu.name.clone(),
                        json_buf: String::new(),
                    },
                    other => AccumulatingBlock::Other(other.clone()),
                };
                self.content_blocks.push(block);
            }
            RawMessageStreamEvent::ContentBlockDelta { index, delta } => {
                if let Some(block) = self.content_blocks.get_mut(*index) {
                    match (block, delta) {
                        (AccumulatingBlock::Text { text, .. }, ContentBlockDelta::TextDelta { text: t }) => {
                            text.push_str(t);
                        }
                        (AccumulatingBlock::Text { citations, .. }, ContentBlockDelta::CitationsDelta { citation }) => {
                            citations.push(citation.clone());
                        }
                        (AccumulatingBlock::Thinking { thinking, .. }, ContentBlockDelta::ThinkingDelta { thinking: t }) => {
                            thinking.push_str(t);
                        }
                        (AccumulatingBlock::Thinking { signature, .. }, ContentBlockDelta::SignatureDelta { signature: s }) => {
                            signature.push_str(s);
                        }
                        (AccumulatingBlock::ToolUse { json_buf, .. }, ContentBlockDelta::InputJsonDelta { partial_json }) => {
                            json_buf.push_str(partial_json);
                        }
                        _ => {} // Mismatched delta/block — ignore
                    }
                }
            }
            RawMessageStreamEvent::ContentBlockStop { .. } => {
                // Nothing to do — block is already accumulated
            }
            RawMessageStreamEvent::MessageDelta { delta, usage } => {
                if let Some(msg) = &mut self.message {
                    if let Some(sr) = &delta.stop_reason {
                        msg.stop_reason = Some(sr.clone());
                    }
                    if let Some(ss) = &delta.stop_sequence {
                        msg.stop_sequence = Some(ss.clone());
                    }
                    msg.usage.output_tokens = usage.output_tokens;
                }
            }
            RawMessageStreamEvent::MessageStop => {}
            RawMessageStreamEvent::Ping => {}
            RawMessageStreamEvent::Error { error } => {
                return Err(AnthropicError::Sse(format!(
                    "{}: {}",
                    error.error_type, error.message
                )));
            }
        }
        Ok(())
    }

    /// Finalize and return the accumulated message.
    pub fn finish(mut self) -> Result<Message> {
        let mut message = self
            .message
            .take()
            .ok_or_else(|| AnthropicError::InvalidData("no message_start event received".into()))?;

        message.content = self
            .content_blocks
            .into_iter()
            .map(|block| match block {
                AccumulatingBlock::Text { text, citations } => {
                    ContentBlock::Text(TextBlock {
                        text,
                        citations: if citations.is_empty() { None } else { Some(citations) },
                    })
                }
                AccumulatingBlock::Thinking { thinking, signature } => {
                    ContentBlock::Thinking(ThinkingBlock { thinking, signature })
                }
                AccumulatingBlock::ToolUse { id, name, json_buf } => {
                    let input = serde_json::from_str(&json_buf)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    ContentBlock::ToolUse(ToolUseBlock { id, name, input })
                }
                AccumulatingBlock::Other(block) => block,
            })
            .collect();

        Ok(message)
    }
}

impl Default for MessageAccumulator {
    fn default() -> Self {
        Self::new()
    }
}
