mod qdrant_client;

use async_trait::async_trait;
use data_structures::{intermediate::Chunk, mock::MockVector};
pub use qdrant_client::QdrantClient;

#[derive(thiserror::Error, Debug)]
pub enum VectorClientError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("No vectors present")]
    Empty,
    #[error("Field missing: {0}")]
    FieldMissing(String),
}

#[async_trait]
pub trait VectorClient {
    const COLLECTION_NAME_PARAGRAPH: &'static str;
    const COLLECTION_NAME_MOCK: &'static str;

    async fn store_chunk(&self, chunk: Chunk) -> Result<(), VectorClientError>;
    async fn get_first_chunk(&self) -> Result<Chunk, VectorClientError>;
    async fn get_all_chunks(&self) -> Result<Vec<Chunk>, VectorClientError>;
}

pub trait VectorPoint<T> {
    fn to_point() -> T;
}
