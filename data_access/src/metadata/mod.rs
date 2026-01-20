use std::{collections::HashMap, pin::Pin};

mod sqlite_client;
pub use sqlite_client::{SqliteClient, SqliteConfig};

#[derive(thiserror::Error, Debug)]
pub enum MetadataClientError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("No vectors present")]
    Empty,
    #[error("Field missing: {0}")]
    FieldMissing(String),
    #[error("Invalid vector dimension: {0}")]
    InvalidVectorDimension(String),
}

pub trait MetadataClient {
    fn store_idf<'a>(
        &'a self,
        map: HashMap<String, f32>,
    ) -> Pin<Box<dyn Future<Output = Result<(), MetadataClientError>> + Send + 'a>>;

    fn load_idf<'a>(
        &'a self,
        run: String,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<String, f32>, MetadataClientError>> + Send + 'a>>;
}
