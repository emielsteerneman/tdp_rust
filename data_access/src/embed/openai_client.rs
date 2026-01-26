use std::future::Future;
use std::pin::Pin;

use async_openai::{
    Client, config::OpenAIConfig as AsyncOpenAIConfig, types::CreateEmbeddingRequestArgs,
};
use serde::Deserialize;
use std::sync::Mutex;
use tracing::info;

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
    total_costs: Mutex<f64>,
}

impl OpenAIClient {
    pub fn new(config: &OpenAiConfig) -> Self {
        let client_config = AsyncOpenAIConfig::new().with_api_key(&config.api_key);
        let client = Client::with_config(client_config);
        OpenAIClient {
            client,
            model_name: config.model_name.clone(),
            total_costs: Mutex::new(0.0),
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

    pub fn get_total_cost(&self) -> f64 {
        *self.total_costs.lock().unwrap()
    }
}

impl EmbedClient for OpenAIClient {
    fn embed_string<'a>(
        &'a self,
        text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            let vecs = self.embed_strings(vec![text.to_string()]).await?;
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
        strings: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            if strings.is_empty() {
                return Ok(Vec::new());
            }

            // The input must not exceed the max input tokens for the model (8192 tokens for `text-embedding-ada-002`),
            // cannot be an empty string,
            // and any array must be 2048 dimensions or less
            let batches = strings.chunks(100);

            if 1 < batches.len() {
                info!("Processing {} batches", batches.len());
            }

            let mut embeddings = Vec::with_capacity(strings.len());

            for batch_strings in batches {
                let request = CreateEmbeddingRequestArgs::default()
                    .model(&self.model_name)
                    .input(batch_strings.to_vec())
                    .build()?;

                let response = self.client.embeddings().create(request).await?;

                let tokens_used = response.usage.prompt_tokens;
                let cost = OpenAIClient::cost_in_cents(&self.model_name, tokens_used);

                {
                    let mut total = self.total_costs.lock().unwrap();
                    *total += cost as f64;

                    println!(
                        "Embedded strings using {} tokens, cost: ${:.6}, total cost so far: ${:.6}",
                        tokens_used, cost, *total
                    );
                }

                let batch_embeddings = response
                    .data
                    .into_iter()
                    .map(|data| data.embedding)
                    .collect::<Vec<Vec<f32>>>();

                embeddings.extend(batch_embeddings);
            }

            Ok(embeddings)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore]
    async fn test_embed_strings() -> Result<(), Box<dyn std::error::Error>> {
        let api_key =
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-placeholder".to_string());
        let openai_config = OpenAiConfig {
            model_name: "text-embedding-3-small".to_string(),
            api_key,
        };

        let client = OpenAIClient::new(&openai_config);
        let strings = vec!["hello".to_string(), "world".to_string()];
        let embeddings = client.embed_strings(strings).await?;
        assert_eq!(embeddings.len(), 2);
        Ok(())
    }
}
