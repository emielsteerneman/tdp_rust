use data_processing::search::Searcher;
use data_structures::embed_type::EmbedType;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    info!("Running Search By Sentence");

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

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

    let query = "battery capacity tigers smallsize";
    let results = searcher
        .search(query.to_string(), Some(5), None, EmbedType::HYBRID)
        .await?;

    for (i, scored_chunk) in results.chunks.iter().enumerate() {
        println!(
            "[{i:2}] {:.4} - {} - {} - {}",
            scored_chunk.score,
            scored_chunk.chunk.league.name_pretty,
            scored_chunk.chunk.team.name_pretty,
            scored_chunk.chunk.year
        );
        println!("{}", scored_chunk.chunk.text);
    }

    if !results.suggestions.teams.is_empty() {
        println!("\nDo you want to filter on one of these teams?");
        for m in &results.suggestions.teams {
            println!("* {m}");
        }
    }

    if !results.suggestions.leagues.is_empty() {
        println!("\nDo you want to filter on one of these leagues?");
        for m in &results.suggestions.leagues {
            println!("* {m}");
        }
    }
    Ok(())
}
