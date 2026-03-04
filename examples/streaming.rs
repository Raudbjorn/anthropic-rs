use anthropic_rs::{Anthropic, MessageCreateParams, Model};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 1024)
        .user("Write a haiku about Rust programming.")
        .build();

    let mut stream = client.messages_create_stream(params).await?;

    while let Some(event) = stream.next().await {
        let event = event?;
        match event {
            anthropic_rs::StreamEvent::ContentBlockDelta {
                delta: anthropic_rs::streaming::events::ContentBlockDelta::TextDelta { text },
                ..
            } => {
                print!("{text}");
            }
            anthropic_rs::StreamEvent::MessageStop => {
                println!();
            }
            _ => {}
        }
    }

    Ok(())
}
