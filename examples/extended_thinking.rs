use anthropic_rs::{Anthropic, MessageCreateParams, Model, ThinkingConfig};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 16000)
        .user("What is the result of 127 * 349? Think through this step by step.")
        .thinking(ThinkingConfig::Enabled { budget_tokens: 10000 })
        .build();

    let message = client.messages_create(params).await?;

    let thinking = message.thinking();
    if !thinking.is_empty() {
        println!("=== Thinking ===");
        println!("{thinking}");
        println!();
    }

    println!("=== Response ===");
    println!("{}", message.text());

    Ok(())
}
