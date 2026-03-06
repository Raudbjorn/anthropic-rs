#![cfg(feature = "realtime")]

use anthropic_rs::realtime::*;

// ── Helper ───────────────────────────────────────────────────────────

fn roundtrip_client(event: &ClientEvent) {
    let json = serde_json::to_string(event).expect("serialize");
    let parsed: ClientEvent = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&parsed).expect("re-serialize");
    assert_eq!(json, json2, "roundtrip mismatch");
}

fn roundtrip_server(json: &str) {
    let parsed: ServerEvent = serde_json::from_str(json).expect("deserialize");
    let reserialized = serde_json::to_string(&parsed).expect("re-serialize");
    let reparsed: ServerEvent = serde_json::from_str(&reserialized).expect("re-deserialize");
    let reserialized2 = serde_json::to_string(&reparsed).expect("re-re-serialize");
    assert_eq!(reserialized, reserialized2, "roundtrip mismatch");
}

// ── Client Event Tests ───────────────────────────────────────────────

#[test]
fn client_session_update() {
    let event = ClientEvent::session_update(Session {
        voice: Some(Voice::Marin),
        instructions: Some("Be helpful.".into()),
        temperature: Some(0.8),
        ..Default::default()
    });
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"session.update""#));
    assert!(json.contains(r#""voice":"marin""#));
}

#[test]
fn client_session_update_with_turn_detection() {
    let event = ClientEvent::session_update(Session {
        turn_detection: Some(TurnDetection::ServerVad {
            threshold: Some(0.5),
            prefix_padding_ms: Some(300),
            silence_duration_ms: Some(500),
            create_response: Some(true),
            interrupt_response: Some(true),
        }),
        ..Default::default()
    });
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"server_vad""#));
    assert!(json.contains(r#""threshold":0.5"#));
}

#[test]
fn client_session_update_with_semantic_vad() {
    let event = ClientEvent::session_update(Session {
        turn_detection: Some(TurnDetection::SemanticVad {
            eagerness: Some(Eagerness::High),
            create_response: None,
            interrupt_response: None,
        }),
        ..Default::default()
    });
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"semantic_vad""#));
    assert!(json.contains(r#""eagerness":"high""#));
}

#[test]
fn client_audio_append() {
    let event = ClientEvent::audio_append("SGVsbG8gV29ybGQ=");
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"input_audio_buffer.append""#));
    assert!(json.contains(r#""audio":"SGVsbG8gV29ybGQ=""#));
}

#[test]
fn client_audio_commit() {
    let event = ClientEvent::audio_commit();
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"input_audio_buffer.commit""#));
}

#[test]
fn client_audio_clear() {
    let event = ClientEvent::audio_clear();
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"input_audio_buffer.clear""#));
}

#[test]
fn client_user_message() {
    let event = ClientEvent::user_message("Hello, how are you?");
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"conversation.item.create""#));
    assert!(json.contains(r#""type":"message""#));
    assert!(json.contains(r#""role":"user""#));
    assert!(json.contains("Hello, how are you?"));
}

#[test]
fn client_create_item_function_output() {
    let item = ConversationItem::function_call_output("call_123", r#"{"result":"sunny"}"#);
    let event = ClientEvent::create_item(item);
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"function_call_output""#));
    assert!(json.contains(r#""call_id":"call_123""#));
}

#[test]
fn client_delete_item() {
    let event = ClientEvent::delete_item("item_abc");
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"conversation.item.delete""#));
    assert!(json.contains(r#""item_id":"item_abc""#));
}

#[test]
fn client_truncate_item() {
    let event = ClientEvent::truncate_item("item_xyz", 0, 1500);
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"conversation.item.truncate""#));
    assert!(json.contains(r#""audio_end_ms":1500"#));
}

#[test]
fn client_create_response() {
    let event = ClientEvent::create_response();
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"response.create""#));
}

#[test]
fn client_create_response_with_params() {
    let params = ResponseCreateParams {
        modalities: Some(vec![Modality::Text]),
        conversation: Some("none".into()),
        metadata: Some(serde_json::json!({"topic": "test"})),
        instructions: Some("Be brief.".into()),
        ..Default::default()
    };
    let event = ClientEvent::create_response_with(params);
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""conversation":"none""#));
    assert!(json.contains(r#""topic":"test""#));
}

#[test]
fn client_cancel_response() {
    let event = ClientEvent::cancel_response();
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""type":"response.cancel""#));
}

#[test]
fn client_event_with_event_id() {
    let event = ClientEvent::create_response().with_event_id("my_event_123");
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""event_id":"my_event_123""#));
}

#[test]
fn client_session_with_tools() {
    let tool = RealtimeTool::function(
        "get_weather",
        "Get the weather for a location",
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": { "type": "string" }
            },
            "required": ["location"]
        }),
    );
    let event = ClientEvent::session_update(Session {
        tools: Some(vec![tool]),
        tool_choice: Some("auto".into()),
        ..Default::default()
    });
    roundtrip_client(&event);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""name":"get_weather""#));
    assert!(json.contains(r#""tool_choice":"auto""#));
}

#[test]
fn client_max_output_tokens_inf() {
    let event = ClientEvent::session_update(Session {
        max_response_output_tokens: Some(MaxOutputTokens::Inf),
        ..Default::default()
    });
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""max_response_output_tokens":"inf""#));
    roundtrip_client(&event);
}

#[test]
fn client_max_output_tokens_limit() {
    let event = ClientEvent::session_update(Session {
        max_response_output_tokens: Some(MaxOutputTokens::Limit(4096)),
        ..Default::default()
    });
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains(r#""max_response_output_tokens":4096"#));
    roundtrip_client(&event);
}

// ── Server Event Tests ───────────────────────────────────────────────

#[test]
fn server_session_created() {
    let json = r#"{
        "type": "session.created",
        "event_id": "evt_001",
        "session": {
            "id": "sess_abc123",
            "model": "gpt-realtime",
            "voice": "marin",
            "modalities": ["text", "audio"],
            "temperature": 0.8
        }
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.event_id(), "evt_001");
    if let ServerEvent::SessionCreated { session, .. } = &event {
        assert_eq!(session.id.as_deref(), Some("sess_abc123"));
        assert_eq!(session.voice, Some(Voice::Marin));
    } else {
        panic!("expected SessionCreated");
    }
}

#[test]
fn server_session_updated() {
    let json = r#"{
        "type": "session.updated",
        "event_id": "evt_002",
        "session": {
            "voice": "coral",
            "instructions": "Be nice"
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_error() {
    let json = r#"{
        "type": "error",
        "event_id": "evt_err",
        "error": {
            "type": "invalid_request_error",
            "code": "invalid_value",
            "message": "Invalid value for 'type'",
            "param": "type",
            "event_id": "my_event"
        }
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert!(event.is_error());
}

#[test]
fn server_conversation_created() {
    let json = r#"{
        "type": "conversation.created",
        "event_id": "evt_conv",
        "conversation": {
            "id": "conv_001",
            "object": "realtime.conversation"
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_conversation_item_created() {
    let json = r#"{
        "type": "conversation.item.created",
        "event_id": "evt_item",
        "item": {
            "id": "item_001",
            "object": "realtime.item",
            "type": "message",
            "role": "user",
            "status": "completed",
            "content": [{"type": "input_text", "text": "Hello"}]
        },
        "previous_item_id": null
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    if let ServerEvent::ConversationItemCreated { item, .. } = &event {
        assert_eq!(item.role, Some(Role::User));
    } else {
        panic!("expected ConversationItemCreated");
    }
}

#[test]
fn server_conversation_item_deleted() {
    let json = r#"{
        "type": "conversation.item.deleted",
        "event_id": "evt_del",
        "item_id": "item_002"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_conversation_item_truncated() {
    let json = r#"{
        "type": "conversation.item.truncated",
        "event_id": "evt_trunc",
        "item_id": "item_003",
        "content_index": 0,
        "audio_end_ms": 1500
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_buffer_committed() {
    let json = r#"{
        "type": "input_audio_buffer.committed",
        "event_id": "evt_commit",
        "item_id": "item_004",
        "previous_item_id": "item_003"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_buffer_cleared() {
    let json = r#"{
        "type": "input_audio_buffer.cleared",
        "event_id": "evt_clear"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_buffer_speech_started() {
    let json = r#"{
        "type": "input_audio_buffer.speech_started",
        "event_id": "evt_speech",
        "item_id": "item_005",
        "audio_start_ms": 500
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_buffer_speech_stopped() {
    let json = r#"{
        "type": "input_audio_buffer.speech_stopped",
        "event_id": "evt_stop",
        "item_id": "item_005",
        "audio_end_ms": 2000
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_transcription_completed() {
    let json = r#"{
        "type": "conversation.item.input_audio_transcription.completed",
        "event_id": "evt_trans",
        "item_id": "item_006",
        "content_index": 0,
        "transcript": "Hello world"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_input_audio_transcription_failed() {
    let json = r#"{
        "type": "conversation.item.input_audio_transcription.failed",
        "event_id": "evt_trans_fail",
        "item_id": "item_007",
        "content_index": 0,
        "error": {
            "type": "transcription_error",
            "code": "audio_unintelligible",
            "message": "Could not transcribe audio"
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_created() {
    let json = r#"{
        "type": "response.created",
        "event_id": "evt_resp",
        "response": {
            "id": "resp_001",
            "object": "realtime.response",
            "status": "in_progress",
            "output": []
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_done() {
    let json = r#"{
        "type": "response.done",
        "event_id": "evt_resp_done",
        "response": {
            "id": "resp_001",
            "object": "realtime.response",
            "status": "completed",
            "output": [{
                "id": "item_008",
                "object": "realtime.item",
                "type": "function_call",
                "status": "completed",
                "name": "get_weather",
                "call_id": "call_001",
                "arguments": "{\"location\":\"San Francisco\"}"
            }],
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "total_tokens": 150
            }
        }
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert!(event.is_response_done());
    let response = event.into_response().unwrap();
    assert_eq!(response.status, Some(ResponseStatus::Completed));
    let output = response.output.unwrap();
    assert_eq!(output[0].name.as_deref(), Some("get_weather"));
}

#[test]
fn server_response_output_item_added() {
    let json = r#"{
        "type": "response.output_item.added",
        "event_id": "evt_oi_add",
        "response_id": "resp_001",
        "output_index": 0,
        "item": {
            "id": "item_009",
            "type": "message",
            "role": "assistant",
            "status": "in_progress"
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_content_part_added() {
    let json = r#"{
        "type": "response.content_part.added",
        "event_id": "evt_cp_add",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "part": {
            "type": "text",
            "text": ""
        }
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_text_delta() {
    let json = r#"{
        "type": "response.text.delta",
        "event_id": "evt_td",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "delta": "Hello"
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.text_delta(), Some("Hello"));
}

#[test]
fn server_response_text_done() {
    let json = r#"{
        "type": "response.text.done",
        "event_id": "evt_td_done",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "text": "Hello, how can I help?"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_audio_delta() {
    let json = r#"{
        "type": "response.audio.delta",
        "event_id": "evt_ad",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "delta": "SGVsbG8="
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.audio_delta(), Some("SGVsbG8="));
}

#[test]
fn server_response_audio_done() {
    let json = r#"{
        "type": "response.audio.done",
        "event_id": "evt_ad_done",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_audio_transcript_delta() {
    let json = r#"{
        "type": "response.audio_transcript.delta",
        "event_id": "evt_atd",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "delta": "Hello"
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.audio_transcript_delta(), Some("Hello"));
}

#[test]
fn server_response_audio_transcript_done() {
    let json = r#"{
        "type": "response.audio_transcript.done",
        "event_id": "evt_atd_done",
        "response_id": "resp_001",
        "item_id": "item_009",
        "output_index": 0,
        "content_index": 0,
        "transcript": "Hello, how can I help you today?"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_response_function_call_arguments_delta() {
    let json = r#"{
        "type": "response.function_call_arguments.delta",
        "event_id": "evt_fca",
        "response_id": "resp_001",
        "item_id": "item_010",
        "output_index": 0,
        "call_id": "call_001",
        "delta": "{\"location\":"
    }"#;
    roundtrip_server(json);

    let event: ServerEvent = serde_json::from_str(json).unwrap();
    assert_eq!(
        event.function_call_arguments_delta(),
        Some("{\"location\":")
    );
}

#[test]
fn server_response_function_call_arguments_done() {
    let json = r#"{
        "type": "response.function_call_arguments.done",
        "event_id": "evt_fca_done",
        "response_id": "resp_001",
        "item_id": "item_010",
        "output_index": 0,
        "call_id": "call_001",
        "arguments": "{\"location\":\"San Francisco\"}"
    }"#;
    roundtrip_server(json);
}

#[test]
fn server_rate_limits_updated() {
    let json = r#"{
        "type": "rate_limits.updated",
        "event_id": "evt_rl",
        "rate_limits": [
            {"name": "requests", "limit": 100, "remaining": 99, "reset_seconds": 60.0},
            {"name": "tokens", "limit": 50000, "remaining": 49500, "reset_seconds": 1.0}
        ]
    }"#;
    roundtrip_server(json);
}

// ── Shared Type Tests ────────────────────────────────────────────────

#[test]
fn voice_serialization() {
    assert_eq!(serde_json::to_string(&Voice::Marin).unwrap(), r#""marin""#);
    assert_eq!(serde_json::to_string(&Voice::Coral).unwrap(), r#""coral""#);

    let custom = Voice::Other("custom_voice".into());
    assert_eq!(
        serde_json::to_string(&custom).unwrap(),
        r#""custom_voice""#
    );
}

#[test]
fn voice_deserialization() {
    let v: Voice = serde_json::from_str(r#""alloy""#).unwrap();
    assert_eq!(v, Voice::Alloy);

    let v: Voice = serde_json::from_str(r#""unknown_voice""#).unwrap();
    assert_eq!(v, Voice::Other("unknown_voice".into()));
}

#[test]
fn audio_format_roundtrip() {
    let formats = [AudioFormat::Pcm16, AudioFormat::G711Ulaw, AudioFormat::G711Alaw];
    for fmt in &formats {
        let json = serde_json::to_string(fmt).unwrap();
        let parsed: AudioFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(*fmt, parsed);
    }
    assert_eq!(serde_json::to_string(&AudioFormat::Pcm16).unwrap(), r#""pcm16""#);
    assert_eq!(serde_json::to_string(&AudioFormat::G711Ulaw).unwrap(), r#""g711_ulaw""#);
}

#[test]
fn realtime_model_roundtrip() {
    let models = [
        RealtimeModel::GptRealtime,
        RealtimeModel::Gpt4oRealtimePreview,
        RealtimeModel::Gpt4oMiniRealtimePreview,
        RealtimeModel::Other("custom-model".into()),
    ];
    for model in &models {
        let json = serde_json::to_string(model).unwrap();
        let parsed: RealtimeModel = serde_json::from_str(&json).unwrap();
        assert_eq!(model.as_str(), parsed.as_str());
    }
}

#[test]
fn max_output_tokens_roundtrip() {
    let inf: MaxOutputTokens = serde_json::from_str(r#""inf""#).unwrap();
    assert_eq!(inf, MaxOutputTokens::Inf);

    let limit: MaxOutputTokens = serde_json::from_str("4096").unwrap();
    assert_eq!(limit, MaxOutputTokens::Limit(4096));
}

#[test]
fn content_part_constructors() {
    let text = ContentPart::input_text("hello");
    assert_eq!(text.content_type, Some(ContentType::InputText));
    assert_eq!(text.text.as_deref(), Some("hello"));

    let audio = ContentPart::input_audio("base64data");
    assert_eq!(audio.content_type, Some(ContentType::InputAudio));
    assert_eq!(audio.audio.as_deref(), Some("base64data"));

    let image = ContentPart::input_image("data:image/png;base64,abc");
    assert_eq!(image.content_type, Some(ContentType::InputImage));
    assert_eq!(image.image_url.as_deref(), Some("data:image/png;base64,abc"));

    let reference = ContentPart::item_reference("item_123");
    assert_eq!(reference.content_type, Some(ContentType::ItemReference));
    assert_eq!(reference.id.as_deref(), Some("item_123"));
}

#[test]
fn conversation_item_constructors() {
    let user_msg = ConversationItem::user_message("Hi");
    assert_eq!(user_msg.item_type, Some(ItemType::Message));
    assert_eq!(user_msg.role, Some(Role::User));

    let sys_msg = ConversationItem::system_message("You are helpful");
    assert_eq!(sys_msg.role, Some(Role::System));

    let fn_output = ConversationItem::function_call_output("call_1", "result");
    assert_eq!(fn_output.item_type, Some(ItemType::FunctionCallOutput));
    assert_eq!(fn_output.call_id.as_deref(), Some("call_1"));
    assert_eq!(fn_output.output.as_deref(), Some("result"));
}

#[test]
fn tracing_auto() {
    let t = Tracing::auto();
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(json, r#""auto""#);
}

#[test]
fn tracing_config() {
    let t = Tracing::Config(TracingConfig {
        group_id: Some("grp_1".into()),
        workflow_name: Some("my_flow".into()),
        metadata: None,
    });
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("grp_1"));
    assert!(json.contains("my_flow"));
}

#[test]
fn realtime_tool_function() {
    let tool = RealtimeTool::function(
        "search",
        "Search for items",
        serde_json::json!({"type": "object"}),
    );
    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains(r#""type":"function""#));
    assert!(json.contains(r#""name":"search""#));

    let parsed: RealtimeTool = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name.as_deref(), Some("search"));
}

// ── Full Lifecycle Simulation ────────────────────────────────────────

#[test]
fn full_text_conversation_lifecycle() {
    // 1. Server sends session.created
    let session_created = r#"{
        "type": "session.created",
        "event_id": "evt_001",
        "session": {"id": "sess_1", "model": "gpt-realtime", "voice": "marin"}
    }"#;
    let event: ServerEvent = serde_json::from_str(session_created).unwrap();
    assert!(matches!(event, ServerEvent::SessionCreated { .. }));

    // 2. Client sends conversation.item.create
    let client_msg = ClientEvent::user_message("What is 2+2?");
    let _json = serde_json::to_string(&client_msg).unwrap();

    // 3. Client sends response.create
    let create_resp = ClientEvent::create_response_with(ResponseCreateParams {
        modalities: Some(vec![Modality::Text]),
        ..Default::default()
    });
    let _json = serde_json::to_string(&create_resp).unwrap();

    // 4. Server sends response.text.delta
    let text_delta = r#"{
        "type": "response.text.delta",
        "event_id": "evt_010",
        "response_id": "resp_1",
        "item_id": "item_1",
        "output_index": 0,
        "content_index": 0,
        "delta": "2+2 equals 4"
    }"#;
    let event: ServerEvent = serde_json::from_str(text_delta).unwrap();
    assert_eq!(event.text_delta(), Some("2+2 equals 4"));

    // 5. Server sends response.done
    let resp_done = r#"{
        "type": "response.done",
        "event_id": "evt_011",
        "response": {
            "id": "resp_1",
            "status": "completed",
            "output": [{
                "id": "item_1",
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": [{"type": "text", "text": "2+2 equals 4"}]
            }],
            "usage": {"input_tokens": 10, "output_tokens": 5, "total_tokens": 15}
        }
    }"#;
    let event: ServerEvent = serde_json::from_str(resp_done).unwrap();
    assert!(event.is_response_done());
}

#[test]
fn full_function_calling_lifecycle() {
    // 1. Client sets up session with tools
    let tool = RealtimeTool::function(
        "get_weather",
        "Get weather for a location",
        serde_json::json!({
            "type": "object",
            "properties": {"location": {"type": "string"}},
            "required": ["location"]
        }),
    );
    let session_update = ClientEvent::session_update(Session {
        tools: Some(vec![tool]),
        tool_choice: Some("auto".into()),
        ..Default::default()
    });
    let _json = serde_json::to_string(&session_update).unwrap();

    // 2. Server responds with function call
    let resp_done = r#"{
        "type": "response.done",
        "event_id": "evt_020",
        "response": {
            "id": "resp_2",
            "status": "completed",
            "output": [{
                "id": "item_fc",
                "type": "function_call",
                "status": "completed",
                "name": "get_weather",
                "call_id": "call_abc",
                "arguments": "{\"location\":\"Paris\"}"
            }]
        }
    }"#;
    let event: ServerEvent = serde_json::from_str(resp_done).unwrap();
    let response = event.into_response().unwrap();
    let item = &response.output.unwrap()[0];
    assert_eq!(item.name.as_deref(), Some("get_weather"));
    assert_eq!(item.call_id.as_deref(), Some("call_abc"));

    // 3. Client sends function_call_output
    let output_item = ConversationItem::function_call_output("call_abc", r#"{"temp":"22C"}"#);
    let output_event = ClientEvent::create_item(output_item);
    let _json = serde_json::to_string(&output_event).unwrap();

    // 4. Client triggers another response
    let create_resp = ClientEvent::create_response();
    let _json = serde_json::to_string(&create_resp).unwrap();
}
