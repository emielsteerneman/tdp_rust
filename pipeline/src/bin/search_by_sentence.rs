use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    info!("Running Search By Sentence");

    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;

    let query = "battery capacity";
    let embedding = embed_client.embed_string(query).await?;

    vector_client
        .search_chunks_by_embedding(embedding, 3)
        .await?;

    Ok(())
}
