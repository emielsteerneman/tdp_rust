mod fastembed_client;
mod openai_client;
pub use fastembed_client::{FastembedClient, FastEmbedConfig};
pub use openai_client::{OpenAIClient, OpenAiConfig};

use async_openai::error::OpenAIError;
use std::pin::Pin;

#[derive(thiserror::Error, Debug)]
pub enum EmbedClientError {
    #[error("Internal client error: {0}")]
    Internal(String),
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Internal client error: {0}")]
    Any(#[from] anyhow::Error),
    #[error("OpenAI client error: {0}")]
    OpenAI(#[from] OpenAIError),
}

pub trait EmbedClient {
    fn embed_string<'a>(
        &'a mut self,
        string: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>>;

    fn embed_strings<'a>(
        &'a mut self,
        strings: Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>>;
}
