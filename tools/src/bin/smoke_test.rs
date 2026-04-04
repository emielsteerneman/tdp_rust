use api::search::{search, SearchArgs};
use data_structures::embed_type::EmbedType;
use data_structures::file::League;
use event_processing::EventSource;
use data_processing::search::Searcher;
use std::collections::HashSet;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);
    let dispatcher = configuration::helpers::build_event_dispatcher(&config);

    let idf_map = Arc::new(metadata_client.load_idf().await?);

    let tdps = metadata_client.load_tdps().await?;
    let mut teams: Vec<String> = tdps
        .iter()
        .map(|tdp| tdp.team_name.name_pretty.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    teams.sort();

    let mut leagues: Vec<String> = tdps
        .iter()
        .map(|tdp| tdp.league.name_pretty().to_string())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    leagues.sort();

    // Collect unique (league, year) pairs from all indexed papers
    let mut league_years: Vec<(League, u32)> = tdps
        .iter()
        .map(|tdp| (tdp.league, tdp.year))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    league_years.sort_by_key(|(l, y)| (l.name().to_string(), *y));

    let searcher = Searcher::new(
        embed_client,
        vector_client,
        metadata_client.clone(),
        idf_map,
        teams,
        leagues,
        config.data_processing.highlight_idf_threshold(),
    );

    println!(
        "Smoke testing {} (league, year) combinations across {} search types\n",
        league_years.len(),
        3
    );

    let search_types = ["sparse", "dense", "hybrid"];

    let mut total = 0;
    let mut failed = 0;

    for (league, year) in &league_years {
        for type_name in &search_types {
            total += 1;

            let search_type = match *type_name {
                "sparse" => EmbedType::SPARSE,
                "dense" => EmbedType::DENSE,
                _ => EmbedType::HYBRID,
            };

            let args = SearchArgs {
                query: "robot".to_string(),
                limit: Some(3),
                league_filter: Some(league.name_pretty().to_string()),
                year_filter: Some(year.to_string()),
                team_filter: None,
                paper_lyt_filter: None,
                content_type_filter: None,
                search_type,
            };

            let label = format!(
                "{:<30} {:>4}  {:<7}",
                league.name_pretty(),
                year,
                type_name
            );

            match search(&searcher, args, &dispatcher, EventSource::Web).await {
                Ok(result) => {
                    let n = result.chunks.len();
                    if n == 0 {
                        println!("  FAIL  {}  0 results", label);
                        failed += 1;
                    } else {
                        // Verify results actually match the requested filters
                        let wrong_league = result
                            .chunks
                            .iter()
                            .any(|c| c.league != *league);
                        let wrong_year = result.chunks.iter().any(|c| c.year != *year);
                        if wrong_league || wrong_year {
                            println!(
                                "  FAIL  {}  {} results but filter mismatch (league={} year={})",
                                label, n, wrong_league, wrong_year
                            );
                            failed += 1;
                        } else {
                            println!("  ok    {}  {} results", label, n);
                        }
                    }
                }
                Err(e) => {
                    println!("  ERR   {}  {}", label, e);
                    failed += 1;
                }
            }
        }
    }

    println!("\n{}/{} passed", total - failed, total);
    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
