use anthropic_rs::{Anthropic, AnthropicError};

#[test]
fn client_from_env_without_key_fails() {
    // Temporarily clear the env var
    let original = std::env::var("ANTHROPIC_API_KEY").ok();
    std::env::remove_var("ANTHROPIC_API_KEY");

    let result = Anthropic::from_env();
    assert!(result.is_err());
    match result.unwrap_err() {
        AnthropicError::Config(msg) => assert!(msg.contains("API key")),
        other => panic!("expected Config error, got: {other}"),
    }

    // Restore
    if let Some(key) = original {
        std::env::set_var("ANTHROPIC_API_KEY", key);
    }
}

#[test]
fn client_builder_with_key() {
    let client = Anthropic::builder()
        .api_key("sk-test-key")
        .base_url("https://custom.api.example.com")
        .build();
    assert!(client.is_ok());
}

#[test]
fn client_debug_omits_key() {
    let client = Anthropic::builder()
        .api_key("sk-secret-key-12345")
        .build()
        .unwrap();
    let debug = format!("{client:?}");
    assert!(!debug.contains("sk-secret-key-12345"));
    assert!(debug.contains("Anthropic"));
}
