use super::AppConfig;
use data_access::{
    embed::{EmbedClient, FastembedClient, OpenAIClient},
    metadata::{MetadataClient, SqliteClient},
    vector::{QdrantClient, VectorClient},
};
use tracing::info;

pub fn load_any_embed_client(config: &AppConfig) -> Box<dyn EmbedClient> {
    // Initialize embed client based on config
    let embed_client: Box<dyn EmbedClient> =
        if let Some(openai_cfg) = &config.data_access.embed.openai {
            info!(
                "Using OpenAI Embeddings with model: {}",
                openai_cfg.model_name
            );
            Box::new(OpenAIClient::new(openai_cfg))
        } else if let Some(fastembed_cfg) = &config.data_access.embed.fastembed {
            info!("Using FastEmbed with model: {}", fastembed_cfg.model_name);
            Box::new(FastembedClient::new(fastembed_cfg).unwrap())
        } else {
            panic!("No embedding configuration found in config.toml");
        };

    embed_client
}

pub async fn load_any_vector_client(
    config: &AppConfig,
) -> Result<Box<dyn VectorClient>, Box<dyn std::error::Error>> {
    // Initialize vector client based on config
    let vector_client: Box<dyn VectorClient> =
        if let Some(qdrant_cfg) = &config.data_access.vector.qdrant {
            info!("Using Qdrant");
            let client = QdrantClient::new(qdrant_cfg.clone()).await?;
            Box::new(client)
        } else {
            panic!("No vector configuration found in config.toml");
        };

    Ok(vector_client)
}

pub fn load_any_metadata_client(config: &AppConfig) -> Box<dyn MetadataClient> {
    // Initialize metadata client based on config
    let metadata_client: Box<dyn MetadataClient> =
        if let Some(sqlite_cfg) = &config.data_access.metadata.sqlite {
            info!("Using SQLite Metadata with file: {}", sqlite_cfg.filename);
            Box::new(SqliteClient::new(sqlite_cfg.clone()))
        } else {
            panic!("No metadata configuration found in config.toml");
        };

    metadata_client
}
