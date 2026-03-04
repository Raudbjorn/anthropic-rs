use anthropic_rs::streaming::accumulator::MessageAccumulator;
use anthropic_rs::streaming::events::{
    ContentBlockDelta, MessageDeltaBody, RawMessageStreamEvent, StreamError,
};
use anthropic_rs::types::content_block::ContentBlock;
use anthropic_rs::types::message::Message;
use anthropic_rs::types::stop_reason::StopReason;
use anthropic_rs::types::text::TextBlock;
use anthropic_rs::types::tool_use::ToolUseBlock;
use anthropic_rs::types::usage::{MessageDeltaUsage, Usage};
use serde_json::json;

fn make_message_start() -> RawMessageStreamEvent {
    RawMessageStreamEvent::MessageStart {
        message: Message {
            id: "msg_test".into(),
            message_type: "message".into(),
            role: "assistant".into(),
            content: vec![],
            model: anthropic_rs::types::model::Model::ClaudeSonnet4_6,
            stop_reason: None,
            stop_sequence: None,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 0,
                ..Default::default()
            },
            container: None,
        },
    }
}

#[test]
fn accumulator_basic_text() {
    let mut acc = MessageAccumulator::new();

    // message_start
    acc.process(&make_message_start()).unwrap();

    // content_block_start
    acc.process(&RawMessageStreamEvent::ContentBlockStart {
        index: 0,
        content_block: ContentBlock::Text(TextBlock {
            text: String::new(),
            citations: None,
        }),
    })
    .unwrap();

    // text deltas
    acc.process(&RawMessageStreamEvent::ContentBlockDelta {
        index: 0,
        delta: ContentBlockDelta::TextDelta {
            text: "Hello".into(),
        },
    })
    .unwrap();

    acc.process(&RawMessageStreamEvent::ContentBlockDelta {
        index: 0,
        delta: ContentBlockDelta::TextDelta {
            text: ", world!".into(),
        },
    })
    .unwrap();

    // content_block_stop
    acc.process(&RawMessageStreamEvent::ContentBlockStop { index: 0 })
        .unwrap();

    // message_delta
    acc.process(&RawMessageStreamEvent::MessageDelta {
        delta: MessageDeltaBody {
            stop_reason: Some(StopReason::EndTurn),
            stop_sequence: None,
            container: None,
        },
        usage: MessageDeltaUsage {
            output_tokens: 5,
            input_tokens: None,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            server_tool_use: None,
        },
    })
    .unwrap();

    // message_stop
    acc.process(&RawMessageStreamEvent::MessageStop).unwrap();

    let msg = acc.finish().unwrap();
    assert_eq!(msg.text(), "Hello, world!");
    assert_eq!(msg.stop_reason, Some(StopReason::EndTurn));
    assert_eq!(msg.usage.output_tokens, 5);
}

#[test]
fn accumulator_tool_use() {
    let mut acc = MessageAccumulator::new();
    acc.process(&make_message_start()).unwrap();

    acc.process(&RawMessageStreamEvent::ContentBlockStart {
        index: 0,
        content_block: ContentBlock::ToolUse(ToolUseBlock {
            id: "tu_123".into(),
            name: "get_weather".into(),
            input: json!({}),
        }),
    })
    .unwrap();

    acc.process(&RawMessageStreamEvent::ContentBlockDelta {
        index: 0,
        delta: ContentBlockDelta::InputJsonDelta {
            partial_json: r#"{"loc"#.into(),
        },
    })
    .unwrap();

    acc.process(&RawMessageStreamEvent::ContentBlockDelta {
        index: 0,
        delta: ContentBlockDelta::InputJsonDelta {
            partial_json: r#"ation":"Paris"}"#.into(),
        },
    })
    .unwrap();

    acc.process(&RawMessageStreamEvent::ContentBlockStop { index: 0 })
        .unwrap();

    acc.process(&RawMessageStreamEvent::MessageDelta {
        delta: MessageDeltaBody {
            stop_reason: Some(StopReason::ToolUse),
            stop_sequence: None,
            container: None,
        },
        usage: MessageDeltaUsage {
            output_tokens: 20,
            input_tokens: None,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            server_tool_use: None,
        },
    })
    .unwrap();

    let msg = acc.finish().unwrap();
    let tools = msg.tool_uses();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "get_weather");
    assert_eq!(tools[0].input["location"], "Paris");
}

#[test]
fn accumulator_error_event() {
    let mut acc = MessageAccumulator::new();
    acc.process(&make_message_start()).unwrap();

    let result = acc.process(&RawMessageStreamEvent::Error {
        error: StreamError {
            error_type: "overloaded_error".into(),
            message: "service is overloaded".into(),
        },
    });
    assert!(result.is_err());
}

#[test]
fn accumulator_no_message_start() {
    let acc = MessageAccumulator::new();
    let result = acc.finish();
    assert!(result.is_err());
}

#[test]
fn stream_event_deserialize_message_start() {
    let json = json!({
        "type": "message_start",
        "message": {
            "id": "msg_abc",
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": "claude-sonnet-4-6",
            "usage": {"input_tokens": 5, "output_tokens": 0}
        }
    });
    let event: RawMessageStreamEvent = serde_json::from_value(json).unwrap();
    match event {
        RawMessageStreamEvent::MessageStart { message } => {
            assert_eq!(message.id, "msg_abc");
        }
        _ => panic!("expected MessageStart"),
    }
}

#[test]
fn stream_event_deserialize_content_block_delta() {
    let json = json!({
        "type": "content_block_delta",
        "index": 0,
        "delta": {
            "type": "text_delta",
            "text": "Hello"
        }
    });
    let event: RawMessageStreamEvent = serde_json::from_value(json).unwrap();
    match event {
        RawMessageStreamEvent::ContentBlockDelta { index, delta } => {
            assert_eq!(index, 0);
            match delta {
                ContentBlockDelta::TextDelta { text } => assert_eq!(text, "Hello"),
                _ => panic!("expected TextDelta"),
            }
        }
        _ => panic!("expected ContentBlockDelta"),
    }
}

#[test]
fn stream_event_deserialize_ping() {
    let json = json!({"type": "ping"});
    let event: RawMessageStreamEvent = serde_json::from_value(json).unwrap();
    assert!(matches!(event, RawMessageStreamEvent::Ping));
}
