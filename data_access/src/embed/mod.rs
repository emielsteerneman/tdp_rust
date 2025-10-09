mod fastembed_client;

pub use fastembed_client::FastembedClient;

#[derive(thiserror::Error, Debug)]
pub enum EmbedClientError {
    #[error("Internal client error: {0}")]
    Internal(String),
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Internal client error: {0}")]
    Any(#[from] anyhow::Error),
}

pub trait EmbedClient {
    fn embed_string(&mut self, string: &str) -> Result<Vec<f32>, EmbedClientError>;
}
