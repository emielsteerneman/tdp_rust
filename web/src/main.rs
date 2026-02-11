use std::net::SocketAddr;
use std::sync::Arc;

mod dto;
mod error;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = "0.0.0.0:8081".parse()?;

    println!("🚀 Web Server initializing...");

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    println!("Config loaded.");

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    println!("Clients initialized.");

    // Load IDF
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
        .map(|tdp| tdp.league.name_pretty.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    leagues.sort();

    let searcher = Searcher::new(
        embed_client.clone(),
        vector_client.clone(),
        Arc::new(idf_map),
        teams,
        leagues,
    );

    let state = AppState::new(metadata_client.clone(), Arc::new(searcher));

    let router = routes::create_router(state);

    println!("🌐 Web Server running on http://0.0.0.0:8081");

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
