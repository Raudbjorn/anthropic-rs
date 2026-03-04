use anthropic_rs::{Anthropic, MessageCreateParams, Model};
use anthropic_rs::batches::{BatchCreateParams, BatchRequest};

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let requests: Vec<BatchRequest> = (1..=3)
        .map(|i| BatchRequest {
            custom_id: format!("request-{i}"),
            params: MessageCreateParams::builder(Model::ClaudeHaiku4_5, 256)
                .user(format!("What is {i} + {i}?"))
                .build(),
        })
        .collect();

    let batch = client
        .batches_create(BatchCreateParams { requests })
        .await?;

    println!("Batch created: {}", batch.id);
    println!("Status: {:?}", batch.processing_status);
    println!("Requests: {:?}", batch.request_counts);

    // Poll for completion
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let batch = client.batches_retrieve(&batch.id).await?;
        println!("Status: {:?}", batch.processing_status);
        if batch.processing_status == anthropic_rs::batches::ProcessingStatus::Ended {
            println!("Batch complete!");
            break;
        }
    }

    Ok(())
}
