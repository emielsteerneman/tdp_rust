use std::pin::Pin;

use crate::embed::{EmbedClient, EmbedClientError};
use fastembed::{
    EmbeddingModel, InitOptions, InitOptionsUserDefined, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FastEmbedConfig {
    pub model_name: String,
}

pub struct FastembedClient {
    model: TextEmbedding,
}

fn init_read_file(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, EmbedClientError> {
    std::fs::read(path).map_err(|e| EmbedClientError::Initialization(e.to_string()))
}

// https://crates.io/crates/fastembed
impl FastembedClient {
    pub fn new(config: &FastEmbedConfig) -> Result<Self, EmbedClientError> {
        // Map string to enum. This is a simple manual mapping.
        // fastembed doesn't seem to export a FromStr for EmbeddingModel easily visible here.
        let model_enum = match config.model_name.as_str() {
            "BGEBaseENV15Q" => EmbeddingModel::BGEBaseENV15Q,
            "AllMiniLML6V2" => EmbeddingModel::AllMiniLML6V2,
            // Add other models as needed
            _ => {
                return Err(EmbedClientError::Initialization(format!(
                    "Unknown or unsupported model name: {}",
                    config.model_name
                )));
            }
        };

        let model = TextEmbedding::try_new(
            InitOptions::new(model_enum)
                .with_cache_dir("/home/emiel/projects/fastembed_cache".into()) // ideally also config
                .with_show_download_progress(true),
        )?;

        Ok(Self { model })
    }

    pub fn new_with_custom_model() -> Result<Self, EmbedClientError> {
        let folder = "/home/emiel/projects/fastembed_cache/models--qdrant--bge-base-en-v1.5-onnx-q/snapshots/738cad1c108e2f23649db9e44b2eab988626493b";

        let onnx_file = init_read_file(format!("{folder}/model_optimized.onnx"))?;
        let tokenizer_files = TokenizerFiles {
            config_file: init_read_file(format!("{folder}/config.json"))?,
            special_tokens_map_file: init_read_file(format!("{folder}/special_tokens_map.json"))?,
            tokenizer_config_file: init_read_file(format!("{folder}/tokenizer_config.json"))?,
            tokenizer_file: init_read_file(format!("{folder}/tokenizer.json"))?,
        };

        let udem = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files);
        let options = InitOptionsUserDefined::new();

        let model = TextEmbedding::try_new_from_user_defined(udem, options)?;

        Ok(FastembedClient { model })
    }
}

impl EmbedClient for FastembedClient {
    fn embed_string<'a>(
        &'a mut self,
        string: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, EmbedClientError>> + Send + 'a>> {
        Box::pin(async move {
            let vecs = self.model.embed(vec![string], None)?;

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
            let vecs = self.model.embed(strings, None)?;
            Ok(vecs)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::{
        EmbedClient,
        fastembed_client::{FastEmbedConfig, FastembedClient},
    };

    #[tokio::test]
    async fn test_initialization() -> Result<(), anyhow::Error> {
        let config = FastEmbedConfig {
            model_name: "BGEBaseENV15Q".to_string(),
        };
        let mut client = FastembedClient::new(&config)?;

        let string = "Hello World!";

        let vec = client.embed_string(string).await?;

        println!("{:?}", vec);

        Ok(())
    }
}
