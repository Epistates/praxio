use praxio::PraxioServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing - write to STDERR for MCP compliance (stdout is for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)  // Critical: logs to stderr, not stdout
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("🚀 Starting Praxio MCP server");

    // Create server
    let server = PraxioServer::new().await;

    // Run with STDIO transport
    tracing::info!("📡 Running on STDIO transport");
    server.run_stdio().await?;

    Ok(())
}
