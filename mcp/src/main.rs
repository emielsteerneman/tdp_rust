use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use std::sync::Arc;

mod server;
mod state;
mod tools;

use server::AppServer;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = "0.0.0.0:8002".parse()?;

    println!("🚀 MCP Server initializing...");

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    println!("Config loaded.");

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    println!("Clients initialized.");

    // Load IDF
    let qdrant_config = config
        .data_access
        .vector
        .qdrant
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Qdrant config is missing"))?;

    let idf_map = metadata_client.load_idf(qdrant_config.run.clone()).await?;

    println!("IDF loaded.");

    let state = AppState::new(
        Arc::from(embed_client),
        Arc::from(vector_client),
        Arc::from(metadata_client),
        idf_map,
    );

    let server = AppServer::new(state);

    println!("🔎 MCP Server running on http://0.0.0.0:8002/mcp");

    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl-c");
        })
        .await?;

    Ok(())
}
