//! Text conversation with the OpenAI Realtime API via WebSocket.
//!
//! Connects, sends a text message, and prints the streaming text response.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --features realtime --example realtime_text
//! ```

use std::io::Write;

use anthropic_rs::realtime::*;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    // Connect using OPENAI_API_KEY from the environment.
    let config = RealtimeConfig::from_env(RealtimeModel::Gpt4oRealtimePreview)?;
    let mut client = RealtimeClient::connect(config).await?;

    // Wait for session.created (always the first server event).
    match client.recv().await {
        Some(Ok(ServerEvent::SessionCreated { session, .. })) => {
            println!(
                "Session created: {}",
                session.id.as_deref().unwrap_or("unknown")
            );
        }
        Some(Ok(ServerEvent::Error { error, .. })) => {
            eprintln!("Server error during setup: {}", error.message);
            client.close().await?;
            return Err(anthropic_rs::AnthropicError::InvalidData(error.message));
        }
        Some(Ok(other)) => {
            eprintln!("Unexpected first event: {other:?}");
            client.close().await?;
            return Ok(());
        }
        Some(Err(e)) => return Err(e),
        None => {
            eprintln!("Connection closed before session was created");
            return Ok(());
        }
    }

    // Configure session for text-only output.
    client
        .update_session(Session {
            modalities: Some(vec![Modality::Text]),
            instructions: Some("You are a helpful assistant. Be concise.".into()),
            ..Default::default()
        })
        .await?;

    // Wait for session.updated confirmation.
    match client.recv().await {
        Some(Ok(ServerEvent::SessionUpdated { .. })) => {
            println!("Session configured for text-only output.");
        }
        Some(Ok(ServerEvent::Error { error, .. })) => {
            return Err(anthropic_rs::AnthropicError::InvalidData(error.message));
        }
        Some(Ok(other)) => eprintln!("Unexpected event after session update: {other:?}"),
        Some(Err(e)) => return Err(e),
        None => return Ok(()),
    }

    // Send a user message and request a response.
    // send_text() creates the conversation item and triggers response.create.
    client
        .send_text("What are the three laws of robotics?")
        .await?;

    println!("\nAssistant: ");

    // Stream the response, printing text deltas as they arrive.
    loop {
        match client.recv().await {
            Some(Ok(event)) => match &event {
                ServerEvent::ResponseTextDelta { delta, .. } => {
                    print!("{delta}");
                    std::io::stdout().flush().unwrap();
                }
                ServerEvent::ResponseDone { response, .. } => {
                    println!();
                    if let Some(usage) = &response.usage {
                        println!(
                            "\nTokens: {} input, {} output",
                            usage.input_tokens.unwrap_or(0),
                            usage.output_tokens.unwrap_or(0),
                        );
                    }
                    break;
                }
                ServerEvent::Error { error, .. } => {
                    eprintln!("\nError: {}", error.message);
                    break;
                }
                _ => {} // Skip other lifecycle events.
            },
            Some(Err(e)) => {
                eprintln!("\nConnection error: {e}");
                break;
            }
            None => break,
        }
    }

    client.close().await?;
    Ok(())
}
