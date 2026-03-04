use anthropic_rs::types::content_block::ContentBlock;
use anthropic_rs::types::content_block_param::ContentBlockParam;
use anthropic_rs::types::message::{Message, MessageParam};
use anthropic_rs::types::model::Model;
use anthropic_rs::types::stop_reason::StopReason;
use anthropic_rs::types::thinking::ThinkingConfig;
use anthropic_rs::types::tool::ToolUnion;
use anthropic_rs::types::tool_choice::ToolChoice;
use anthropic_rs::types::usage::Usage;
use anthropic_rs::types::web_search::WebSearchTool;
use serde_json::json;

#[test]
fn model_roundtrip() {
    let model = Model::ClaudeSonnet4_6;
    let json = serde_json::to_string(&model).unwrap();
    assert_eq!(json, r#""claude-sonnet-4-6""#);
    let back: Model = serde_json::from_str(&json).unwrap();
    assert_eq!(back, model);
}

#[test]
fn model_other_roundtrip() {
    let model = Model::Other("custom-model-v1".into());
    let json = serde_json::to_string(&model).unwrap();
    assert_eq!(json, r#""custom-model-v1""#);
    let back: Model = serde_json::from_str(&json).unwrap();
    assert_eq!(back, model);
}

#[test]
fn stop_reason_roundtrip() {
    let sr = StopReason::EndTurn;
    let json = serde_json::to_string(&sr).unwrap();
    assert_eq!(json, r#""end_turn""#);
    let back: StopReason = serde_json::from_str(&json).unwrap();
    assert_eq!(back, sr);
}

#[test]
fn message_param_user() {
    let msg = MessageParam::user("Hello");
    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["role"], "user");
    assert_eq!(json["content"], "Hello");
}

#[test]
fn content_block_text_deserialize() {
    let json = json!({
        "type": "text",
        "text": "Hello, world!"
    });
    let block: ContentBlock = serde_json::from_value(json).unwrap();
    assert_eq!(block.as_text(), Some("Hello, world!"));
}

#[test]
fn content_block_tool_use_deserialize() {
    let json = json!({
        "type": "tool_use",
        "id": "tu_123",
        "name": "get_weather",
        "input": {"location": "Paris"}
    });
    let block: ContentBlock = serde_json::from_value(json).unwrap();
    let tu = block.as_tool_use().unwrap();
    assert_eq!(tu.name, "get_weather");
    assert_eq!(tu.input["location"], "Paris");
}

#[test]
fn content_block_thinking_deserialize() {
    let json = json!({
        "type": "thinking",
        "thinking": "Let me think...",
        "signature": "sig123"
    });
    let block: ContentBlock = serde_json::from_value(json).unwrap();
    assert_eq!(block.as_thinking(), Some("Let me think..."));
}

#[test]
fn content_block_param_text_serialize() {
    let param = ContentBlockParam::text("Hello");
    let json = serde_json::to_value(&param).unwrap();
    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "Hello");
}

#[test]
fn tool_choice_variants() {
    let auto = ToolChoice::auto();
    let json = serde_json::to_value(&auto).unwrap();
    assert_eq!(json["type"], "auto");

    let tool = ToolChoice::tool("get_weather");
    let json = serde_json::to_value(&tool).unwrap();
    assert_eq!(json["type"], "tool");
    assert_eq!(json["name"], "get_weather");

    let none = ToolChoice::none();
    let json = serde_json::to_value(&none).unwrap();
    assert_eq!(json["type"], "none");
}

#[test]
fn thinking_config_roundtrip() {
    let config = ThinkingConfig::Enabled { budget_tokens: 5000 };
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["type"], "enabled");
    assert_eq!(json["budget_tokens"], 5000);
    let back: ThinkingConfig = serde_json::from_value(json).unwrap();
    match back {
        ThinkingConfig::Enabled { budget_tokens } => assert_eq!(budget_tokens, 5000),
        _ => panic!("expected Enabled"),
    }
}

#[test]
fn tool_union_custom_tool_roundtrip() {
    let tool = anthropic_rs::types::tool::Tool::new(
        "my_tool",
        json!({"type": "object", "properties": {}}),
    )
    .with_description("A test tool");

    let union = ToolUnion::Custom(tool);
    let json = serde_json::to_value(&union).unwrap();
    assert_eq!(json["name"], "my_tool");

    let back: ToolUnion = serde_json::from_value(json).unwrap();
    match back {
        ToolUnion::Custom(t) => {
            assert_eq!(t.name, "my_tool");
            assert_eq!(t.description.as_deref(), Some("A test tool"));
        }
        _ => panic!("expected Custom"),
    }
}

#[test]
fn tool_union_web_search_roundtrip() {
    let ws = WebSearchTool::new();
    let union = ToolUnion::WebSearch(ws);
    let json = serde_json::to_value(&union).unwrap();
    assert_eq!(json["type"], "web_search_20260209");

    let back: ToolUnion = serde_json::from_value(json).unwrap();
    match back {
        ToolUnion::WebSearch(t) => assert_eq!(t.name, "web_search"),
        _ => panic!("expected WebSearch"),
    }
}

#[test]
fn usage_deserialize() {
    let json = json!({
        "input_tokens": 100,
        "output_tokens": 50,
        "cache_creation_input_tokens": 10,
        "cache_read_input_tokens": 5
    });
    let usage: Usage = serde_json::from_value(json).unwrap();
    assert_eq!(usage.input_tokens, 100);
    assert_eq!(usage.output_tokens, 50);
    assert_eq!(usage.cache_creation_input_tokens, Some(10));
}

#[test]
fn message_deserialize() {
    let json = json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "model": "claude-sonnet-4-6",
        "content": [
            {"type": "text", "text": "Hello!"}
        ],
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5
        }
    });
    let msg: Message = serde_json::from_value(json).unwrap();
    assert_eq!(msg.id, "msg_123");
    assert_eq!(msg.model, Model::ClaudeSonnet4_6);
    assert_eq!(msg.text(), "Hello!");
    assert_eq!(msg.stop_reason, Some(StopReason::EndTurn));
}

#[test]
fn message_create_params_serialize() {
    let params = anthropic_rs::MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("Hello")
        .system("You are helpful.")
        .temperature(0.7)
        .build();

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["model"], "claude-sonnet-4-6");
    assert_eq!(json["max_tokens"], 1024);
    assert_eq!(json["messages"][0]["role"], "user");
    assert_eq!(json["system"], "You are helpful.");
    assert!((json["temperature"].as_f64().unwrap() - 0.7).abs() < f64::EPSILON);
    assert!(json.get("stream").is_none());
}

#[test]
fn web_search_tool_result_deserialize() {
    let json = json!({
        "type": "web_search_tool_result",
        "tool_use_id": "tu_456",
        "content": [
            {
                "type": "web_search_result",
                "url": "https://example.com",
                "title": "Example",
                "encrypted_content": "abc123"
            }
        ]
    });
    let block: ContentBlock = serde_json::from_value(json).unwrap();
    match block {
        ContentBlock::WebSearchToolResult(r) => {
            assert_eq!(r.tool_use_id, "tu_456");
            assert_eq!(r.content.len(), 1);
            assert_eq!(r.content[0].title, "Example");
        }
        _ => panic!("expected WebSearchToolResult"),
    }
}
