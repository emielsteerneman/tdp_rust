use super::AppConfig;
use data_access::{
    embed::{EmbedClient, FastembedClient, OpenAIClient},
    metadata::{MetadataClient, SqliteClient},
    teams::{TeamRegistryClient, TeamsSqliteClient},
    vector::{QdrantClient, VectorClient},
};
use event_processing::dispatcher::EventDispatcher;
use event_processing::listeners::sqlite::SqliteListener;
use event_processing::listeners::telegram::TelegramListener;
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

pub fn build_team_registry_client(config: &AppConfig) -> Option<Arc<dyn TeamRegistryClient + Send + Sync>> {
    let teams_config = config.data_access.teams.as_ref()?;
    let sqlite_cfg = teams_config.sqlite.as_ref()?;

    info!("Using SQLite Teams Registry with file: {}", sqlite_cfg.filename);
    Some(Arc::new(TeamsSqliteClient::new(sqlite_cfg.clone())))
}

pub fn build_event_dispatcher(config: &AppConfig) -> Arc<EventDispatcher> {
    let mut dispatcher = EventDispatcher::new();

    if let Some(ref ep_config) = config.event_processing {
        if let Some(ref activity_config) = ep_config.activity {
            if let Some(ref sqlite_cfg) = activity_config.sqlite {
                info!("Registering SQLite event listener: {}", sqlite_cfg.filename);
                match SqliteListener::new(sqlite_cfg) {
                    Ok(listener) => dispatcher.register(Arc::new(listener)),
                    Err(e) => tracing::error!("Failed to create SQLite event listener: {}", e),
                }
            }
        }

        if let Some(ref telegram_cfg) = ep_config.telegram {
            info!("Registering Telegram event listener");
            dispatcher.register(Arc::new(TelegramListener::new(telegram_cfg)));
        }
    }

    Arc::new(dispatcher)
}
