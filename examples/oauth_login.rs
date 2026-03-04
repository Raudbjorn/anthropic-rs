//! Example: OAuth login and API usage.
//!
//! Run with: `cargo run --features oauth --example oauth_login`

use std::time::Duration;

use anthropic_rs::oauth::{
    CallbackServer, FileTokenStorage, OAuthBackend, OAuthConfig, OAuthFlow,
};
use anthropic_rs::{Anthropic, MessageCreateParams, Model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if already authenticated
    let storage = FileTokenStorage::default_path()?;
    let flow = OAuthFlow::new(storage);

    if !flow.is_authenticated()? {
        // Need to log in
        let storage = FileTokenStorage::default_path()?;

        // Start callback server on auto-port
        let server = CallbackServer::bind(0).await?;
        let redirect_uri = server.redirect_uri();
        let config = OAuthConfig::default().with_redirect_uri(&redirect_uri);

        let mut flow = OAuthFlow::with_config(storage, config);
        let (auth_url, _state) = flow.start_authorization()?;

        println!("Open this URL to authenticate:\n  {auth_url}\n");

        // Try to open browser
        let _ = open::that(&auth_url);

        println!("Waiting for callback...");
        let callback = server.wait_for_callback(Duration::from_secs(300)).await?;
        flow.exchange_code(&callback.code, callback.state.as_deref())
            .await?;
        println!("Authenticated!");
    } else {
        println!("Already authenticated.");
    }

    // Use the OAuth backend with the Anthropic client
    let storage = FileTokenStorage::default_path()?;
    let backend = OAuthBackend::new(storage);
    let client = Anthropic::builder().backend(backend).build()?;

    // Make an API call
    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 256)
        .user("What is 2 + 2?")
        .build();

    let message = client.messages_create(params).await?;
    println!("\nClaude says: {}", message.text());

    Ok(())
}
