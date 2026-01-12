use data_access::file::utilities::load_from_dir_all_tdp_json;
use data_processing::tdp_to_chunks;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    info!("Running TDPs JSON to Qdrant importer");

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = match configuration::helpers::load_any_vector_client(&config).await {
        Ok(client) => client,
        Err(err) => {
            error!("Could not load a vector client: {err}");
            return;
        }
    };

    let tdps = load_from_dir_all_tdp_json("/home/emiel/projects/tdps_json").unwrap();
    info!("Loaded {} TDPs", tdps.len());

    let tdps = tdps
        .iter()
        .filter(|tdp| tdp.name.league.league_minor == "smallsize")
        .collect::<Vec<_>>();

    for tdp in tdps.iter().take(20) {
        info!("Processing TDP: {}", tdp.name.get_filename());
        let chunks = tdp_to_chunks(tdp, Some(embed_client.as_ref())).await;
        info!("Generated {} chunks. Storing..", chunks.len());
        for chunk in chunks {
            vector_client.store_chunk(chunk).await.unwrap();
        }
    }
}
