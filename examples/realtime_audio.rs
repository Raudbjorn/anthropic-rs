//! Audio interaction with the OpenAI Realtime API.
//!
//! Demonstrates the audio buffer API by sending a synthetic PCM16 tone,
//! collecting the audio response, and printing the transcript.
//!
//! ```bash
//! OPENAI_API_KEY=sk-... cargo run --features realtime --example realtime_audio
//! ```

use std::io::Write;

use anthropic_rs::realtime::{audio, *};

/// Generate a simple 440 Hz sine tone as PCM16 LE bytes at 24 kHz.
fn generate_sine_tone(duration_secs: f64) -> Vec<u8> {
    let sample_rate = 24_000u32;
    let frequency = 440.0_f64;
    let num_samples = (sample_rate as f64 * duration_secs) as usize;
    let mut pcm = Vec::with_capacity(num_samples * 2);

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let sample = (t * frequency * 2.0 * std::f64::consts::PI).sin();
        // Scale to i16 range with reduced amplitude to avoid clipping.
        let value = (sample * 16_000.0) as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }

    pcm
}

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
        Some(Err(e)) => return Err(e),
        _ => {
            eprintln!("Unexpected first event or connection closed.");
            return Ok(());
        }
    }

    // Configure session for audio input/output.
    client
        .update_session(Session {
            modalities: Some(vec![Modality::Text, Modality::Audio]),
            voice: Some(Voice::Marin),
            input_audio_format: Some(AudioFormat::Pcm16),
            output_audio_format: Some(AudioFormat::Pcm16),
            input_audio_transcription: Some(InputAudioTranscription {
                model: Some("whisper-1".into()),
                language: None,
                prompt: None,
            }),
            turn_detection: None,
            instructions: Some(
                "You are a helpful assistant. Respond briefly to any audio you receive.".into(),
            ),
            ..Default::default()
        })
        .await?;

    // Wait for session.updated confirmation.
    match client.recv().await {
        Some(Ok(ServerEvent::SessionUpdated { .. })) => {
            println!("Session configured for audio (voice: Marin, pcm16).");
        }
        Some(Ok(other)) => eprintln!("Unexpected event: {other:?}"),
        Some(Err(e)) => return Err(e),
        None => return Ok(()),
    }

    // Generate a short synthetic tone (0.5 seconds) to demonstrate the audio API.
    // In a real application you would read from a microphone or WAV file.
    let pcm_bytes = generate_sine_tone(0.5);
    println!(
        "Generated {} bytes of PCM16 audio ({:.1}s at 24kHz).",
        pcm_bytes.len(),
        pcm_bytes.len() as f64 / (24_000.0 * 2.0)
    );

    // Encode audio to base64 and send it to the input audio buffer.
    let encoded = audio::encode_audio_base64(&pcm_bytes);
    client
        .send(ClientEvent::audio_append(encoded))
        .await?;

    // Commit the audio buffer and trigger a response.
    client.send(ClientEvent::audio_commit()).await?;
    client.create_response(None).await?;

    println!("\nWaiting for response...\n");

    // Collect response audio and transcript.
    let mut audio_bytes_received: usize = 0;
    let mut transcript = String::new();

    loop {
        match client.recv().await {
            Some(Ok(event)) => match &event {
                ServerEvent::ResponseAudioDelta { delta, .. } => {
                    match audio::decode_audio_base64(delta) {
                        Ok(bytes) => audio_bytes_received += bytes.len(),
                        Err(e) => eprintln!("Audio decode error: {e}"),
                    }
                }
                ServerEvent::ResponseAudioTranscriptDelta { delta, .. } => {
                    print!("{delta}");
                    std::io::stdout().flush().unwrap();
                    transcript.push_str(delta);
                }
                ServerEvent::ResponseDone { response, .. } => {
                    println!();
                    println!("\n--- Response complete ---");
                    println!("Transcript: {transcript}");
                    println!(
                        "Audio received: {} bytes ({:.1}s at 24kHz PCM16)",
                        audio_bytes_received,
                        audio_bytes_received as f64 / (24_000.0 * 2.0)
                    );
                    if let Some(usage) = &response.usage {
                        println!(
                            "Tokens: {} input, {} output",
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
