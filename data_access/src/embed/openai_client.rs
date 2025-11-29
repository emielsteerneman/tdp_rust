use std::pin::Pin;

use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};

use super::EmbedClient;
use super::EmbedClientError;

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
}

impl OpenAIClient {
    pub fn new() -> Self {
        dotenvy::from_filename(".env").expect("Could not load .env file");
        let client = Client::new();
        OpenAIClient { client }
    }

    pub fn cost_in_cents(model: &str, n_tokens: u32) -> f32 {
        // Prices per 1M tokens https://platform.openai.com/docs/pricing
        // text-embedding-3-small : $0.02
        // text-embedding-3-large : $0.13

        match model {
            "text-embedding-3-small" => (0.02 / 1e6) * (n_tokens as f32),
            "text-embedding-3-large" => (0.13 / 1e6) * (n_tokens as f32),
            _ => panic!("Unknown embedding model: {}", model),
        }
    }
}

impl EmbedClient for OpenAIClient {
    fn embed_string<'a>(
        &'a mut self,
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
        &'a mut self,
        strings: Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<f32>>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            if strings.is_empty() {
                return Ok(Vec::new());
            }

            let request = CreateEmbeddingRequestArgs::default()
                .model("text-embedding-3-large")
                .input(strings)
                .build()?;

            let response = self.client.embeddings().create(request).await?;

            let tokens_used = response.usage.prompt_tokens;
            let cost = OpenAIClient::cost_in_cents("text-embedding-3-large", tokens_used);
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

#[cfg(test)]
mod tests {
    use crate::embed::{EmbedClient, openai_client::OpenAIClient};

    #[tokio::test]
    async fn test_initialization() -> Result<(), anyhow::Error> {
        dotenvy::from_filename(".env").expect("Could not load .env file");

        let mut client = OpenAIClient::new();

        client.embed_string("Hello World!").await?;

        client
            .embed_strings(vec![
                "Hello World!",
                "How are you doing?",
                "Would you like some coffee?",
            ])
            .await?;

        Ok(())
    }
}
