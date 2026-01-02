use crate::embed::FastEmbedConfig;
use crate::embed::OpenAiConfig;
use crate::vector::QdrantConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataAccessConfig {
    pub embed: EmbedConfig,
    pub vector: VectorConfig,
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
