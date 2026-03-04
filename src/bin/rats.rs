//! rats — CLI for the Anthropic Rust SDK (AnthThropic-RS → ATRS → rats).
//!
//! Usage:
//!   rats login [--client-id ID] [--no-browser] [--no-qr] [--port PORT]
//!   rats logout
//!   rats status

use std::time::Duration;

use clap::{Parser, Subcommand};

use anthropic_rs::oauth::{
    CallbackServer, FileTokenStorage, OAuthConfig, OAuthFlow, TokenStorage,
};

#[derive(Parser)]
#[command(name = "rats", about = "rats — Anthropic Rust SDK CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate with Anthropic via OAuth.
    Login {
        /// OAuth client ID (defaults to Anthropic's public client).
        #[arg(long)]
        client_id: Option<String>,

        /// Don't open the browser automatically.
        #[arg(long)]
        no_browser: bool,

        /// Don't render QR code in terminal.
        #[arg(long)]
        no_qr: bool,

        /// Port for the local callback server (0 = auto).
        #[arg(long, default_value = "0")]
        port: u16,
    },
    /// Remove stored OAuth tokens.
    Logout,
    /// Show current authentication status.
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Login {
            client_id,
            no_browser,
            no_qr,
            port,
        } => login(client_id, no_browser, no_qr, port).await?,
        Command::Logout => logout()?,
        Command::Status => status()?,
    }

    Ok(())
}

async fn login(
    client_id: Option<String>,
    no_browser: bool,
    no_qr: bool,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileTokenStorage::default_path()?;

    let mut config = OAuthConfig::default();
    if let Some(id) = client_id {
        config = config.with_client_id(id);
    }

    // Start callback server
    let server = CallbackServer::bind(port).await?;
    let redirect_uri = server.redirect_uri();

    // Use local redirect URI
    config = config.with_redirect_uri(&redirect_uri);

    let mut flow = OAuthFlow::with_config(storage, config);
    let (auth_url, _flow_state) = flow.start_authorization()?;

    println!("\nOpen this URL to authenticate:\n");
    println!("  {auth_url}\n");

    // Render QR code if available
    if !no_qr {
        render_qr(&auth_url);
    }

    // Open browser
    if !no_browser {
        if let Err(e) = open::that(&auth_url) {
            eprintln!("Could not open browser: {e}");
            eprintln!("Please open the URL above manually.");
        }
    }

    println!("Waiting for authentication callback...\n");

    // Wait for callback
    let callback = server.wait_for_callback(Duration::from_secs(300)).await?;

    // Exchange code for tokens
    let token = flow
        .exchange_code(&callback.code, callback.state.as_deref())
        .await?;

    println!("Authentication successful!");
    println!("Token expires: {}", token.expires_at_datetime().format("%Y-%m-%d %H:%M:%S UTC"));

    Ok(())
}

fn logout() -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileTokenStorage::default_path()?;
    let flow = OAuthFlow::new(storage);
    flow.logout()?;
    println!("Logged out successfully.");
    Ok(())
}

fn status() -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileTokenStorage::default_path()?;
    let flow = OAuthFlow::new(storage);

    if flow.is_authenticated()? {
        let token = flow.storage().load()?.unwrap();
        let remaining = token.time_until_expiry();
        println!("Authenticated");
        println!("  Token expires: {}", token.expires_at_datetime().format("%Y-%m-%d %H:%M:%S UTC"));
        println!("  Time remaining: {}m {}s", remaining.as_secs() / 60, remaining.as_secs() % 60);
        if token.needs_refresh() {
            println!("  (token will be refreshed on next API call)");
        }
    } else {
        let has_token = flow.storage().load()?.is_some();
        if has_token {
            println!("Token expired. Run `rats login` to re-authenticate.");
        } else {
            println!("Not authenticated. Run `rats login` to authenticate.");
        }
    }

    Ok(())
}

fn render_qr(url: &str) {
    use qrcode::QrCode;
    use qrcode::render::unicode;

    match QrCode::new(url) {
        Ok(code) => {
            let image = code
                .render::<unicode::Dense1x2>()
                .dark_color(unicode::Dense1x2::Light)
                .light_color(unicode::Dense1x2::Dark)
                .build();
            println!("{image}\n");
        }
        Err(e) => {
            eprintln!("Could not generate QR code: {e}");
        }
    }
}
