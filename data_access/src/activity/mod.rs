use std::future::Future;
use std::pin::Pin;

use mockall::automock;

mod sqlite_client;
pub use sqlite_client::{ActivitySqliteClient, ActivitySqliteConfig};

#[derive(thiserror::Error, Debug)]
pub enum ActivityClientError {
    #[error("Internal error: {0}")]
    Internal(String),
}

#[automock]
pub trait ActivityClient: Send + Sync {
    fn log_event<'a>(
        &'a self,
        source: String,
        event_type: String,
        payload: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), ActivityClientError>> + Send + 'a>>;
}
