use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use std::sync::Arc;

mod server;
mod state;

use server::AppServer;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 MCP Server initializing...");

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    println!("Config loaded.");

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let dispatcher = configuration::helpers::build_event_dispatcher(&config);
    let registry = configuration::helpers::build_registry_client(&config);

    metadata_client.print_analytics().await?;

    println!("Clients initialized.");

    let idf_map = metadata_client.load_idf().await?;

    println!("IDF loaded.");

    let tdps = metadata_client.load_tdps().await?;
    use std::collections::HashSet;
    let mut teams: Vec<String> = tdps
        .iter()
        .map(|tdp| tdp.team_name.name_pretty.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    teams.sort();

    use data_processing::search::Searcher;

    let mut leagues: Vec<String> = tdps
        .iter()
        .map(|tdp| tdp.league.name_pretty().to_string())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    leagues.sort();

    let searcher = Searcher::new(
        embed_client.clone(),
        vector_client.clone(),
        metadata_client.clone(),
        Arc::new(idf_map),
        teams,
        leagues,
        config.data_processing.highlight_idf_threshold(),
    );

    let state = AppState::new(metadata_client.clone(), Arc::new(searcher), dispatcher, registry, config.website_url.clone());
    let server = AppServer::new(state);

    // The MCP service is Clone — both routers share the same underlying factory.
    let mcp_service = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // ── Port 50001: open MCP (no auth) ─────────────────────────────────────
    let open_router = axum::Router::new().nest_service("/mcp", mcp_service);
    let open_addr: SocketAddr = "0.0.0.0:50001".parse()?;
    let open_listener = tokio::net::TcpListener::bind(open_addr).await?;

    println!("🔎 MCP Server (open) running on http://0.0.0.0:50001/mcp");

    tokio::select! {
        result = axum::serve(open_listener, open_router) => result?,
        _ = tokio::signal::ctrl_c() => {},
    }

    Ok(())
}
