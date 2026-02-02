mod fastembed_client;
mod openai_client;
pub use fastembed_client::{FastEmbedConfig, FastembedClient};
pub use openai_client::{OpenAIClient, OpenAiConfig};

use async_openai::error::OpenAIError;
use data_structures::IDF;
use data_structures::text_utils::process_text_to_words;
use std::collections::HashMap;
use std::future::Future;
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
        &'a self,
        string: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>>;

    fn embed_strings<'a>(
        &'a self,
        strings: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>>;

    fn embed_sparse(&self, text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
        embed_sparse(text, idf_map)
    }
}

pub fn embed_sparse(text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
    let mut map = HashMap::new();

    let (ngram1, ngram2, ngram3) = process_text_to_words(text);
    let iter = ngram1.iter().chain(ngram2.iter()).chain(ngram3.iter());

    for word in iter {
        if let Some((id, idf)) = idf_map.get(word) {
            *map.entry(*id).or_insert(0.0) += idf;
        }
    }

    map
}
