use api::activity::EventSource;
use api::search::{search_structured, SearchArgs};
use data_processing::search::Searcher;
use data_structures::embed_type::EmbedType;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        eprintln!("Usage: {} <query> [--mode <dense|sparse|hybrid>]", args[0]);
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  <query>                    Search query string");
        eprintln!("  --mode <dense|sparse|hybrid>  Search mode (default: hybrid)");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} \"battery capacity tigers\"", args[0]);
        eprintln!("  {} \"neural network\" --mode dense", args[0]);
        eprintln!("  {} \"bang bang\" --mode sparse", args[0]);
        std::process::exit(1);
    }

    let query = &args[1];

    // Parse search mode from command line (default to hybrid)
    let mut search_mode = EmbedType::HYBRID;
    if let Some(mode_idx) = args.iter().position(|arg| arg == "--mode") {
        if let Some(mode_str) = args.get(mode_idx + 1) {
            search_mode = match mode_str.to_lowercase().as_str() {
                "dense" => EmbedType::DENSE,
                "sparse" => EmbedType::SPARSE,
                "hybrid" => EmbedType::HYBRID,
                _ => {
                    eprintln!("Invalid mode: {}. Use 'dense', 'sparse', or 'hybrid'", mode_str);
                    std::process::exit(1);
                }
            };
        }
    }

    info!("Running Search By Sentence");
    info!("Query: {}", query);
    info!("Mode: {:?}", search_mode);

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let activity_client = configuration::helpers::load_activity_client(&config);

    let idf_map = Arc::new(metadata_client.load_idf().await?);

    let tdps = metadata_client.load_tdps().await?;
    let mut teams = tdps
        .iter()
        .map(|tdp| tdp.team_name.name_pretty.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    teams.sort();

    let mut leagues = tdps
        .into_iter()
        .map(|tdp| tdp.league.name_pretty.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    leagues.sort();

    let searcher = Searcher::new(embed_client, vector_client, idf_map, teams, leagues);

    println!("\n=== Search Results ===");
    println!("Query: {}", query);
    println!("Mode: {:?}", search_mode);

    // Use the API search function with Dev source
    let search_args = SearchArgs {
        query: query.to_string(),
        limit: Some(5),
        league_filter: None,
        year_filter: None,
        team_filter: None,
        lyti_filter: None,
        search_type: search_mode,
    };

    let results = search_structured(&searcher, search_args, activity_client, EventSource::Dev)
        .await?;

    println!("Found {} results\n", results.chunks.len());

    for (i, scored_chunk) in results.chunks.iter().enumerate() {
        println!(
            "[{:2}] Score: {:.4} | {} | {} | {}",
            i,
            scored_chunk.score,
            scored_chunk.chunk.league.name_pretty,
            scored_chunk.chunk.team.name_pretty,
            scored_chunk.chunk.year
        );
        println!("    {}", scored_chunk.chunk.text);
        println!();
    }

    if !results.suggestions.teams.is_empty() {
        println!("Team suggestions:");
        for m in &results.suggestions.teams {
            println!("  * {m}");
        }
        println!();
    }

    if !results.suggestions.leagues.is_empty() {
        println!("League suggestions:");
        for m in &results.suggestions.leagues {
            println!("  * {m}");
        }
    }

    Ok(())
}
