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

#[derive(Debug, Clone)]
pub struct ActivityEvent {
    pub id: i64,
    pub timestamp: String,
    pub source: String,
    pub event_type: String,
    pub payload: Option<String>,
}

#[automock]
pub trait ActivityClient: Send + Sync {
    fn log_event<'a>(
        &'a self,
        source: String,
        event_type: String,
        payload: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), ActivityClientError>> + Send + 'a>>;

    fn query_events<'a>(
        &'a self,
        source: Option<String>,
        event_type: Option<String>,
        since: Option<String>,
        limit: Option<u32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ActivityEvent>, ActivityClientError>> + Send + 'a>>;
}
