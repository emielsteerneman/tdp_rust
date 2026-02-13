use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::Deserialize;

use super::{ActivityClient, ActivityClientError, ActivityEvent};

#[derive(Debug, Deserialize, Clone)]
pub struct ActivitySqliteConfig {
    pub filename: String,
}

pub struct ActivitySqliteClient {
    conn: Arc<Mutex<Connection>>,
}

impl ActivitySqliteClient {
    pub fn new(config: ActivitySqliteConfig) -> Self {
        let conn =
            Connection::open(&config.filename).expect("Failed to open activity SQLite database");
        conn.query_row("PRAGMA journal_mode=WAL;", [], |_| Ok(()))
            .expect("Failed to set WAL mode");

        let client = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        client.ensure_schema();
        client
    }

    fn ensure_schema(&self) {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                source TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);
            CREATE INDEX IF NOT EXISTS idx_events_source ON events (source);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events (event_type);
            ",
        )
        .expect("Failed to create activity schema");
    }
}

impl ActivityClient for ActivitySqliteClient {
    fn log_event<'a>(
        &'a self,
        source: String,
        event_type: String,
        payload: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), ActivityClientError>> + Send + 'a>> {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO events (source, event_type, payload) VALUES (?1, ?2, ?3)",
                    rusqlite::params![source, event_type, payload],
                )
                .map_err(|e| ActivityClientError::Internal(e.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|e| ActivityClientError::Internal(e.to_string()))?
        })
    }
    fn query_events<'a>(
        &'a self,
        source: Option<String>,
        event_type: Option<String>,
        since: Option<String>,
        limit: Option<u32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ActivityEvent>, ActivityClientError>> + Send + 'a>>
    {
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let conn = conn.lock().unwrap();

                let mut sql = String::from("SELECT id, timestamp, source, event_type, payload FROM events WHERE 1=1");
                let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

                if let Some(ref s) = source {
                    sql.push_str(" AND source = ?");
                    params.push(Box::new(s.clone()));
                }
                if let Some(ref et) = event_type {
                    sql.push_str(" AND event_type = ?");
                    params.push(Box::new(et.clone()));
                }
                if let Some(ref s) = since {
                    sql.push_str(" AND timestamp >= ?");
                    params.push(Box::new(s.clone()));
                }

                sql.push_str(" ORDER BY timestamp DESC");

                if let Some(l) = limit {
                    sql.push_str(" LIMIT ?");
                    params.push(Box::new(l));
                }

                let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                    params.iter().map(|p| p.as_ref()).collect();

                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| ActivityClientError::Internal(e.to_string()))?;

                let events = stmt
                    .query_map(param_refs.as_slice(), |row| {
                        Ok(ActivityEvent {
                            id: row.get(0)?,
                            timestamp: row.get(1)?,
                            source: row.get(2)?,
                            event_type: row.get(3)?,
                            payload: row.get(4)?,
                        })
                    })
                    .map_err(|e| ActivityClientError::Internal(e.to_string()))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ActivityClientError::Internal(e.to_string()))?;

                Ok(events)
            })
            .await
            .map_err(|e| ActivityClientError::Internal(e.to_string()))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_client() -> ActivitySqliteClient {
        ActivitySqliteClient::new(ActivitySqliteConfig {
            filename: ":memory:".to_string(),
        })
    }

    #[tokio::test]
    async fn test_log_event() {
        let client = temp_client();
        let result = client
            .log_event("web".into(), "search".into(), r#"{"query":"bang bang"}"#.into())
            .await;
        assert!(result.is_ok());

        let conn = client.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_log_multiple_events() {
        let client = temp_client();
        client
            .log_event("web".into(), "search".into(), r#"{"query":"trajectory"}"#.into())
            .await
            .unwrap();
        client
            .log_event("mcp".into(), "search".into(), r#"{"query":"ball detection"}"#.into())
            .await
            .unwrap();
        client
            .log_event("web".into(), "paper_open".into(), r#"{"paper_id":"ssl__2024__Tigers__0"}"#.into())
            .await
            .unwrap();

        let conn = client.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 3);

        let web_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE source = 'web'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(web_count, 2);
    }

    #[tokio::test]
    async fn test_event_fields_stored() {
        let client = temp_client();
        client
            .log_event("mcp".into(), "list_teams".into(), r#"{"hint":"tiger"}"#.into())
            .await
            .unwrap();

        let conn = client.conn.lock().unwrap();
        let (source, event_type, payload): (String, String, String) = conn
            .query_row(
                "SELECT source, event_type, payload FROM events LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(source, "mcp");
        assert_eq!(event_type, "list_teams");
        assert_eq!(payload, r#"{"hint":"tiger"}"#);
    }

    #[tokio::test]
    async fn test_query_events_with_filters() {
        let client = temp_client();
        client
            .log_event("web".into(), "search".into(), r#"{"query":"trajectory"}"#.into())
            .await
            .unwrap();
        client
            .log_event("mcp".into(), "search".into(), r#"{"query":"ball detection"}"#.into())
            .await
            .unwrap();
        client
            .log_event("web".into(), "paper_open".into(), r#"{"paper_id":"ssl__2024__Tigers__0"}"#.into())
            .await
            .unwrap();

        // All events
        let all = client.query_events(None, None, None, None).await.unwrap();
        assert_eq!(all.len(), 3);

        // Filter by source
        let web_only = client
            .query_events(Some("web".into()), None, None, None)
            .await
            .unwrap();
        assert_eq!(web_only.len(), 2);

        // Filter by event type
        let searches = client
            .query_events(None, Some("search".into()), None, None)
            .await
            .unwrap();
        assert_eq!(searches.len(), 2);

        // Limit
        let limited = client.query_events(None, None, None, Some(1)).await.unwrap();
        assert_eq!(limited.len(), 1);

        // Combined filters
        let web_searches = client
            .query_events(Some("web".into()), Some("search".into()), None, None)
            .await
            .unwrap();
        assert_eq!(web_searches.len(), 1);
        assert_eq!(web_searches[0].source, "web");
        assert_eq!(web_searches[0].event_type, "search");
    }

    #[tokio::test]
    async fn test_query_events_since_filter() {
        let client = temp_client();
        client
            .log_event("web".into(), "search".into(), r#"{"query":"old"}"#.into())
            .await
            .unwrap();

        // Get the timestamp of the event we just inserted, then query with a future timestamp
        let events = client.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 1);
        let ts = &events[0].timestamp;

        // Query with the exact timestamp should include it
        let since_result = client
            .query_events(None, None, Some(ts.clone()), None)
            .await
            .unwrap();
        assert_eq!(since_result.len(), 1);

        // Query with a future timestamp should return nothing
        let future = client
            .query_events(None, None, Some("2099-01-01T00:00:00Z".into()), None)
            .await
            .unwrap();
        assert!(future.is_empty());
    }

    #[tokio::test]
    async fn test_query_events_ordering() {
        let client = temp_client();
        client
            .log_event("web".into(), "search".into(), r#"{"query":"first"}"#.into())
            .await
            .unwrap();
        client
            .log_event("web".into(), "search".into(), r#"{"query":"second"}"#.into())
            .await
            .unwrap();

        let events = client.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 2);
        // DESC order: most recent first
        assert!(events[0].timestamp >= events[1].timestamp);
    }
}
