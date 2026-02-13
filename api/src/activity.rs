use std::sync::Arc;

use data_access::activity::ActivityClient;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Web,
    Mcp,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSource::Web => "web",
            EventSource::Mcp => "mcp",
        }
    }
}

/// Fire-and-forget activity logging. Spawns a task, never blocks the caller.
/// If `client` is `None`, logging is silently skipped.
pub fn log_activity(
    client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
    event_type: &'static str,
    payload: impl Serialize + Send + 'static,
) {
    let Some(client) = client else {
        return;
    };

    let source_str = source.as_str().to_string();
    let event_type_str = event_type.to_string();
    let payload_json = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => {
            tracing::warn!("Failed to serialize activity payload: {}", e);
            return;
        }
    };

    tokio::spawn(async move {
        if let Err(e) = client
            .log_event(source_str, event_type_str, payload_json)
            .await
        {
            tracing::warn!("Failed to log activity event: {}", e);
        }
    });
}
