use serde::Deserialize;
use crate::embed::OpenAiConfig;
use crate::embed::FastEmbedConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct DataAccessConfig {
    pub embed: EmbedConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbedConfig {
    pub openai: Option<OpenAiConfig>,
    pub fastembed: Option<FastEmbedConfig>,
}
