use anthropic_rs::{backends::VertexBackend, Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    // Reads ANTHROPIC_VERTEX_ACCESS_TOKEN, CLOUD_ML_REGION,
    // and ANTHROPIC_VERTEX_PROJECT_ID.
    let backend = VertexBackend::from_env()?;
    let client = Anthropic::builder().backend(backend).build()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("What is the capital of France? Reply in one sentence.")
        .build();

    let message = client.messages_create(params).await?;
    println!("Response: {}", message.text());

    Ok(())
}
