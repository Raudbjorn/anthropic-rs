use anthropic_rs::{Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What is the capital of France? Reply in one sentence.")
        .build();

    let message = client.messages_create(params).await?;
    println!("Response: {}", message.text());
    println!("Usage: {} input, {} output tokens",
        message.usage.input_tokens, message.usage.output_tokens);

    Ok(())
}
