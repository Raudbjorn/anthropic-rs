use anthropic_rs::{Anthropic, MessageCreateParams, Model};
use anthropic_rs::types::web_search::WebSearchTool;

#[tokio::main]
async fn main() -> anthropic_rs::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Anthropic::from_env()?;

    let params = MessageCreateParams::builder(Model::ClaudeSonnet4_6, 4096)
        .user("What are the latest developments in Rust programming language? Search the web.")
        .tool(WebSearchTool::new())
        .build();

    let message = client.messages_create(params).await?;

    for block in &message.content {
        match block {
            anthropic_rs::ContentBlock::Text(t) => println!("{}", t.text),
            anthropic_rs::ContentBlock::ServerToolUse(stu) => {
                println!("[Server tool: {} ({})]", stu.name, stu.id);
            }
            anthropic_rs::ContentBlock::WebSearchToolResult(r) => {
                println!("[Web search results: {} items]", r.content.len());
                for result in &r.content {
                    println!("  - {} ({})", result.title, result.url);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
