use anthropic_rs::{
    Anthropic, ContentBlockParam, MessageCreateParams, MessageParam, Model, Tool, ToolChoice,
};
use serde_json::json;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let weather_tool = Tool::new(
        "get_weather",
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["location"]
        }),
    )
    .with_description("Get the current weather for a location.");

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What's the weather like in Tokyo?")
        .tool(weather_tool)
        .tool_choice(ToolChoice::auto())
        .build();

    let message = client.messages_create(params).await?;

    for tool_use in message.tool_uses() {
        println!("Tool: {}", tool_use.name);
        println!("Input: {}", serde_json::to_string_pretty(&tool_use.input)?);
    }

    // Return tool result
    if let Some(tool_use) = message.tool_uses().first() {
        let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
            .user("What's the weather like in Tokyo?")
            .message(MessageParam::assistant(message.content.iter().map(|b| {
                match b {
                    anthropic_rs::ContentBlock::ToolUse(tu) => ContentBlockParam::ToolUse(
                        anthropic_rs::types::tool_use::ToolUseBlockParam {
                            id: tu.id.clone(),
                            name: tu.name.clone(),
                            input: tu.input.clone(),
                            cache_control: None,
                        },
                    ),
                    anthropic_rs::ContentBlock::Text(t) => ContentBlockParam::text(&t.text),
                    _ => ContentBlockParam::text(""),
                }
            }).collect::<Vec<_>>()))
            .message(MessageParam::user(vec![
                ContentBlockParam::tool_result(&tool_use.id, r#"{"temperature": 22, "condition": "sunny"}"#),
            ]))
            .build();

        let response = client.messages_create(params).await?;
        println!("\nFinal response: {}", response.text());
    }

    Ok(())
}
