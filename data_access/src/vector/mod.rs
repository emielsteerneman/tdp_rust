mod qdrant_client;

use async_trait::async_trait;
use data_structures::intermediate::Chunk;
pub use qdrant_client::{QdrantClient, QdrantConfig};
use uuid::Uuid;

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
    async fn store_chunk(&self, chunk: Chunk) -> Result<(), VectorClientError>;
    async fn get_all_chunks(&self) -> Result<Vec<Chunk>, VectorClientError>;
    async fn get_chunk_by_id(&self, id: Uuid) -> Result<Chunk, VectorClientError>;
}

pub trait VectorPoint<T> {
    fn to_point() -> T;
}
