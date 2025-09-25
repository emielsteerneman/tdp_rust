use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::embed::EmbedClient;

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
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_cache_dir("/home/emiel/Desktop/fastembed_cache".into())
                .with_show_download_progress(true),
        )?;
        Ok(Self { model })
    }
}

impl EmbedClient for FastembedClient {
    fn embed_string(self, string: String) -> Vec<f32> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::fastembed_client::FastembedClient;

    #[tokio::test]
    async fn test_initialization() -> Result<(), anyhow::Error> {
        let client = FastembedClient::new()?;

        Ok(())
    }
}
