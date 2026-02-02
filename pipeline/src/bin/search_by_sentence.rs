use std::collections::HashSet;

use data_processing::utils::match_names;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    info!("Running Search By Sentence");

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    let idf_map = metadata_client.load_idf().await?;

    let tdps = metadata_client.load_tdps(vec![]).await?;
    let mut teams = tdps
        .into_iter()
        .map(|tdp| tdp.team_name.name_pretty)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    teams.sort();

    // for team in teams {
    //     println!("{}", team);
    // }

    let query = "battery capacity er force tigers";
    let dense = embed_client.embed_string(query).await?;
    let sparse = embed_client.embed_sparse(query, &idf_map);

    let team_matches = match_names(teams.clone(), query.to_string());

    let chunks = vector_client
        .search_chunks(Some(dense), Some(sparse), 5, None)
        .await?;

    for (i, (chunk, score)) in chunks.iter().enumerate() {
        println!(
            "[{i:2}] {score:.4} - {} - {} - {}",
            chunk.league.name_pretty, chunk.team.name_pretty, chunk.year
        );
        println!("{}", chunk.text);
    }

    if !team_matches.is_empty() {
        println!("\nDo you want to filter on one of these team?");
        for m in team_matches {
            println!("* {m}");
        }
    }
    Ok(())
}
