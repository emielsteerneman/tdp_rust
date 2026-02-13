use crate::activity::ActivitySqliteConfig;
use crate::embed::FastEmbedConfig;
use crate::embed::OpenAiConfig;
use crate::metadata::SqliteConfig;
use crate::vector::QdrantConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataAccessConfig {
    pub run: String,
    pub embed: EmbedConfig,
    pub vector: VectorConfig,
    pub metadata: MetadataConfig,
    pub activity: Option<ActivityConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActivityConfig {
    pub sqlite: Option<ActivitySqliteConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbedConfig {
    pub openai: Option<OpenAiConfig>,
    pub fastembed: Option<FastEmbedConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VectorConfig {
    pub qdrant: Option<QdrantConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetadataConfig {
    pub sqlite: Option<SqliteConfig>,
}
