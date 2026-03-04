use anthropic_rs::{backends::BedrockBackend, Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    // Reads AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SESSION_TOKEN,
    // and AWS_REGION (defaults to us-east-1).
    let backend = BedrockBackend::from_env()?;
    let client = Anthropic::builder().backend(backend).build()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What is the capital of France? Reply in one sentence.")
        .build();

    let message = client.messages_create(params).await?;
    println!("Response: {}", message.text());

    Ok(())
}
