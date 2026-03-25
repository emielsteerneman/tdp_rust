use crate::embed::FastEmbedConfig;
use crate::embed::OpenAiConfig;
use crate::metadata::SqliteConfig;
use crate::teams::TeamsSqliteConfig;
use crate::vector::QdrantConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataAccessConfig {
    pub embed: EmbedConfig,
    pub vector: VectorConfig,
    pub metadata: MetadataConfig,
    pub teams: Option<TeamsConfig>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct TeamsConfig {
    pub sqlite: Option<TeamsSqliteConfig>,
}
