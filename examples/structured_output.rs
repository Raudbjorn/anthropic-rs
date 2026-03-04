use anthropic_rs::{Anthropic, MessageCreateParams, Model};
use anthropic_rs::types::output_config::OutputConfig;
use serde_json::json;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("List the 3 largest countries by area. Return as JSON with name and area_km2 fields.")
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

    let message = client.messages_create(params).await?;
    let json: serde_json::Value = serde_json::from_str(&message.text())?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
