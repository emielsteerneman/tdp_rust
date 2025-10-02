use crate::embed::{EmbedClient, EmbedClientError};
use data_structures::structure::Sentence;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct FastembedClient {
    model: TextEmbedding,
}

#[derive(thiserror::Error, Debug)]
pub enum FastembedClientError {
    #[error("Internal Fastembed error: {0}")]
    Internal(#[from] fastembed::Error),
}

// https://crates.io/crates/fastembed
impl FastembedClient {
    pub fn new() -> Result<Self, FastembedClientError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGEBaseENV15Q)
                .with_cache_dir("/home/emiel/Desktop/projects/fastembed_cache".into())
                .with_show_download_progress(true),
        )?;

        Ok(Self { model })
    }

    pub fn weird_test_return_sentence() -> Sentence {
        todo!();
    }
}

impl EmbedClient for FastembedClient {
    fn embed_string(&mut self, string: &str) -> Result<Vec<f32>, EmbedClientError> {
        let vecs = self.model.embed(vec![string], Some(1))?;

        let Some(vec) = vecs.into_iter().next() else {
            return Err(EmbedClientError::Internal(
                "No vectors returned".to_string(),
            ));
        };

        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::{EmbedClient, fastembed_client::FastembedClient};

    #[tokio::test]
    async fn test_initialization() -> Result<(), anyhow::Error> {
        let mut client = FastembedClient::new()?;

        let string = "Hello World!";

        let vec = client.embed_string(string)?;

        println!("{:?}", vec);

        Ok(())
    }
}
