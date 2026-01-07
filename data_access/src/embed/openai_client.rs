use std::pin::Pin;

use async_openai::{
    Client, config::OpenAIConfig as AsyncOpenAIConfig, types::CreateEmbeddingRequestArgs,
};
use serde::Deserialize;

use super::EmbedClient;
use super::EmbedClientError;

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiConfig {
    pub model_name: String,
    pub api_key: String, // Explicitly pass API key, don't rely on env var implicit loading
}

pub struct OpenAIClient {
    client: Client<AsyncOpenAIConfig>,
    model_name: String,
}

impl OpenAIClient {
    pub fn new(config: &OpenAiConfig) -> Self {
        let client_config = AsyncOpenAIConfig::new().with_api_key(&config.api_key);
        let client = Client::with_config(client_config);
        OpenAIClient {
            client,
            model_name: config.model_name.clone(),
        }
    }

    pub fn cost_in_cents(model: &str, n_tokens: u32) -> f32 {
        // Prices per 1M tokens https://platform.openai.com/docs/pricing
        // text-embedding-3-small : $0.02
        // text-embedding-3-large : $0.13

        match model {
            "text-embedding-3-small" => (0.02 / 1e6) * (n_tokens as f32),
            "text-embedding-3-large" => (0.13 / 1e6) * (n_tokens as f32),
            _ => 0.0, // Don't panic, just return 0 if unknown
        }
    }
}

impl EmbedClient for OpenAIClient {
    fn embed_string<'a>(
        &'a self,
        text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            let vecs = self.embed_strings(vec![text]).await?;
            let Some(vec) = vecs.into_iter().next() else {
                return Err(EmbedClientError::Internal(
                    "No vectors returned".to_string(),
                ));
            };
            Ok(vec)
        })
    }

    fn embed_strings<'a>(
        &'a self,
        strings: Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            if strings.is_empty() {
                return Ok(Vec::new());
            }

            let request = CreateEmbeddingRequestArgs::default()
                .model(&self.model_name)
                .input(strings)
                .build()?;

            let response = self.client.embeddings().create(request).await?;

            let tokens_used = response.usage.prompt_tokens;
            let cost = OpenAIClient::cost_in_cents(&self.model_name, tokens_used);
            println!(
                "Embedded strings using {} tokens, cost: ${:.6}",
                tokens_used, cost
            );

            #[cfg(test)]
            for data in response.clone().data {
                println!(
                    "[{}]: has embedding of length {}",
                    data.index,
                    data.embedding.len()
                )
            }

            let embeddings = response
                .data
                .into_iter()
                .map(|data| data.embedding)
                .collect::<Vec<Vec<f32>>>();

            Ok(embeddings)
        })
    }
}
