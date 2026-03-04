//! File upload example (requires `beta` feature).
//!
//! Run with: cargo run --example file_upload --features beta

#[cfg(feature = "beta")]
#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = anthropic_rs::Anthropic::from_env()?;

    // Upload a text file
    let content = b"Hello from anthropic-rs!";
    let metadata = client
        .files_upload(
            bytes::Bytes::from_static(content),
            "hello.txt",
            "assistants",
        )
        .await?;

    println!("Uploaded: {} ({})", metadata.filename, metadata.id);
    println!("Size: {} bytes", metadata.size_bytes);

    // Download it back
    let downloaded = client.files_download(&metadata.id).await?;
    println!("Downloaded: {}", String::from_utf8_lossy(&downloaded));

    // List files
    let page = client
        .files_list(anthropic_rs::beta::files::FileListParams::default())
        .await?;
    println!("Files: {}", page.data.len());

    // Clean up
    let deleted = client.files_delete(&metadata.id).await?;
    println!("Deleted: {}", deleted.id);

    Ok(())
}

#[cfg(not(feature = "beta"))]
fn main() {
    eprintln!("This example requires the `beta` feature flag.");
    eprintln!("Run with: cargo run --example file_upload --features beta");
}
