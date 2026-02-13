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

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::activity::MockActivityClient;

    #[test]
    fn test_event_source_as_str() {
        assert_eq!(EventSource::Web.as_str(), "web");
        assert_eq!(EventSource::Mcp.as_str(), "mcp");
    }

    #[tokio::test]
    async fn test_log_activity_none_client_does_not_panic() {
        log_activity(None, EventSource::Web, "search", serde_json::json!({"q": "test"}));
    }

    #[tokio::test]
    async fn test_log_activity_calls_client() {
        let mut mock = MockActivityClient::new();
        mock.expect_log_event()
            .withf(|source, event_type, payload| {
                source == "web"
                    && event_type == "search"
                    && payload.contains("\"q\"")
                    && payload.contains("\"test\"")
            })
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let client: Arc<dyn ActivityClient + Send + Sync> = Arc::new(mock);
        log_activity(Some(client), EventSource::Web, "search", serde_json::json!({"q": "test"}));

        // Give the spawned task time to run
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_log_activity_mcp_source() {
        let mut mock = MockActivityClient::new();
        mock.expect_log_event()
            .withf(|source, _, _| source == "mcp")
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let client: Arc<dyn ActivityClient + Send + Sync> = Arc::new(mock);
        log_activity(Some(client), EventSource::Mcp, "list_teams", serde_json::json!({}));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
