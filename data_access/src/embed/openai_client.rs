use std::pin::Pin;

use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};

use super::EmbedClient;
use super::EmbedClientError;

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
}

impl OpenAIClient {
    pub fn new() -> Self {
        let client = Client::new();
        OpenAIClient { client }
    }
}

impl EmbedClient for OpenAIClient {
    fn embed_string<'a>(
        &'a mut self,
        text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            let request = CreateEmbeddingRequestArgs::default()
                .model("text-embedding-3-large")
                .input([text])
                .build()?;

            let response = self.client.embeddings().create(request).await?;

            for data in response.data {
                println!(
                    "[{}]: has embedding of length {}",
                    data.index,
                    data.embedding.len()
                )
            }

            Ok(vec![])
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

        let string = "Hello World!";

        let vec = client.embed_string(string).await?;

        println!("{:?}", vec);

        Ok(())
    }
}
