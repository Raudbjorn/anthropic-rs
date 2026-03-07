//! Function calling with the OpenAI Realtime API.
//!
//! Registers a weather tool, sends a query that triggers a function call,
//! returns mock weather data, and prints the final text response.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --features realtime --example realtime_function_calling
//! ```

use std::io::Write;

use anthropic_rs::realtime::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    let config = RealtimeConfig::from_env(RealtimeModel::Gpt4oRealtimePreview)?;
    let mut client = RealtimeClient::connect(config).await?;

    // Wait for session.created.
    match client.recv().await {
        Some(Ok(ServerEvent::SessionCreated { session, .. })) => {
            println!(
                "Session created: {}",
                session.id.as_deref().unwrap_or("unknown")
            );
        }
        Some(Ok(ServerEvent::Error { error, .. })) => {
            return Err(anthropic_rs::AnthropicError::InvalidData(error.message));
        }
        Some(Err(e)) => return Err(e),
        _ => {
            eprintln!("Unexpected first event or connection closed.");
            return Ok(());
        }
    }

    // Define a weather tool.
    let weather_tool = RealtimeTool::function(
        "get_weather",
        "Get the current weather for a city.",
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name, e.g. 'Tokyo'"
                }
            },
            "required": ["city"]
        }),
    );

    // Register the tool handler on the client.
    // The handler receives parsed JSON arguments and returns a JSON result.
    client.add_tool(weather_tool.clone(), |args: serde_json::Value| async move {
        let city = args
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        println!("  [tool] get_weather called for: {city}");

        // Return mock weather data.
        json!({
            "city": city,
            "temperature_celsius": 22,
            "condition": "partly cloudy",
            "humidity_percent": 65,
            "wind_kph": 12
        })
    })?;

    // Configure session with text modality, the weather tool, and auto tool choice.
    client
        .update_session(Session {
            modalities: Some(vec![Modality::Text]),
            instructions: Some(
                "You are a helpful weather assistant. Use the get_weather tool \
                 when asked about weather. Report the results clearly."
                    .into(),
            ),
            tools: Some(vec![weather_tool]),
            tool_choice: Some("auto".into()),
            ..Default::default()
        })
        .await?;

    // Wait for session.updated.
    match client.recv().await {
        Some(Ok(ServerEvent::SessionUpdated { .. })) => {
            println!("Session configured with get_weather tool.\n");
        }
        Some(Ok(ServerEvent::Error { error, .. })) => {
            return Err(anthropic_rs::AnthropicError::InvalidData(error.message));
        }
        Some(Ok(other)) => eprintln!("Unexpected event: {other:?}"),
        Some(Err(e)) => return Err(e),
        None => return Ok(()),
    }

    // Ask about the weather so the model invokes the tool.
    // send_text() creates the conversation item and triggers response.create.
    client
        .send_text("What's the weather like in Tokyo right now?")
        .await?;

    println!("Waiting for function call...\n");

    // Event loop: handle the function call round-trip and the final response.
    let mut awaiting_final_response = false;

    loop {
        match client.recv().await {
            Some(Ok(event)) => match &event {
                ServerEvent::ResponseFunctionCallArgumentsDone {
                    call_id, arguments, ..
                } => {
                    println!("  [event] Function call complete (call_id: {call_id})");
                    println!("  [event] Arguments: {arguments}");
                }
                ServerEvent::ResponseDone { response, .. } => {
                    // Check if the response contains function calls that need handling.
                    let has_function_calls = response
                        .output
                        .as_ref()
                        .map(|items| {
                            items
                                .iter()
                                .any(|item| item.item_type == Some(ItemType::FunctionCall))
                        })
                        .unwrap_or(false);

                    if has_function_calls && !awaiting_final_response {
                        println!("\n  [event] Response contains function call(s), handling...\n");

                        // Use handle_function_calls to invoke registered handlers,
                        // submit results, and trigger a new response automatically.
                        client.handle_function_calls(response).await?;

                        awaiting_final_response = true;
                        println!("Assistant: ");
                    } else {
                        // This is the final text response.
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
                }
                ServerEvent::ResponseTextDelta { delta, .. } => {
                    print!("{delta}");
                    std::io::stdout().flush().unwrap();
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
