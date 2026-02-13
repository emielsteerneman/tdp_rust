use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::Deserialize;

use super::{ActivityClient, ActivityClientError};

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
}
