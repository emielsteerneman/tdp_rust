use super::AppConfig;
use data_access::{
    activity::{ActivityClient, ActivitySqliteClient},
    embed::{EmbedClient, FastembedClient, OpenAIClient},
    metadata::{MetadataClient, SqliteClient},
    vector::{QdrantClient, VectorClient},
};
use std::sync::Arc;
use tracing::info;

pub fn load_any_embed_client(config: &AppConfig) -> Arc<dyn EmbedClient + Send + Sync> {
    // Initialize embed client based on config
    let embed_client: Arc<dyn EmbedClient + Send + Sync> =
        if let Some(openai_cfg) = &config.data_access.embed.openai {
            info!(
                "Using OpenAI Embeddings with model: {}",
                openai_cfg.model_name
            );
            Arc::new(OpenAIClient::new(openai_cfg))
        } else if let Some(fastembed_cfg) = &config.data_access.embed.fastembed {
            info!("Using FastEmbed with model: {}", fastembed_cfg.model_name);
            Arc::new(FastembedClient::new(fastembed_cfg).unwrap())
        } else {
            panic!("No embedding configuration found in config.toml");
        };

    embed_client
}

pub async fn load_any_vector_client(
    config: &AppConfig,
) -> anyhow::Result<Arc<dyn VectorClient + Send + Sync>> {
    // Initialize vector client based on config
    let vector_client: Arc<dyn VectorClient + Send + Sync> =
        if let Some(qdrant_cfg) = &config.data_access.vector.qdrant {
            info!("Using Qdrant");
            let client = QdrantClient::new(qdrant_cfg.clone()).await?;
            Arc::new(client)
        } else {
            panic!("No vector configuration found in config.toml");
        };

    Ok(vector_client)
}

pub fn load_any_metadata_client(config: &AppConfig) -> Arc<dyn MetadataClient + Send + Sync> {
    // Initialize metadata client based on config
    let metadata_client: Arc<dyn MetadataClient + Send + Sync> =
        if let Some(sqlite_cfg) = &config.data_access.metadata.sqlite {
            info!("Using SQLite Metadata with file: {}", sqlite_cfg.filename);
            Arc::new(SqliteClient::new(sqlite_cfg.clone()))
        } else {
            panic!("No metadata configuration found in config.toml");
        };

    metadata_client
}

pub fn load_activity_client(
    config: &AppConfig,
) -> Option<Arc<dyn ActivityClient + Send + Sync>> {
    if let Some(activity_config) = &config.data_access.activity {
        if let Some(sqlite_cfg) = &activity_config.sqlite {
            info!(
                "Using SQLite Activity with file: {}",
                sqlite_cfg.filename
            );
            return Some(Arc::new(ActivitySqliteClient::new(sqlite_cfg.clone())));
        }
    }
    info!("No activity client configured, activity logging disabled");
    None
}
