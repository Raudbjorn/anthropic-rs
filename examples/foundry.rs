use anthropic_rs::{backends::FoundryBackend, Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    // Reads ANTHROPIC_FOUNDRY_API_KEY and ANTHROPIC_FOUNDRY_RESOURCE
    // (or ANTHROPIC_FOUNDRY_BASE_URL).
    let backend = FoundryBackend::from_env()?;
    let client = Anthropic::builder().backend(backend).build()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What is the capital of France? Reply in one sentence.")
        .build();

    let message = client.messages_create(params).await?;
    println!("Response: {}", message.text());

    Ok(())
}
