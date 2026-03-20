use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use rusqlite::Connection;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::task;

use crate::{Event, EventListener, EventListenerError, EventSource};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteListenerConfig {
    pub filename: String,
}

// ---------------------------------------------------------------------------
// ActivityEvent (query result)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ActivityEvent {
    pub id: i64,
    pub timestamp: String,
    pub source: String,
    pub event_type: String,
    pub payload: Option<String>,
}

// ---------------------------------------------------------------------------
// SqliteListener
// ---------------------------------------------------------------------------

pub struct SqliteListener {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteListener {
    pub fn new(config: &SqliteListenerConfig) -> Result<Self, EventListenerError> {
        let conn = Connection::open(&config.filename)
            .map_err(EventListenerError::Database)?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(EventListenerError::Database)?;

        Self::ensure_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn ensure_schema(conn: &Connection) -> Result<(), EventListenerError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                source     TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload    TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_events_source     ON events(source);
            CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_events_timestamp  ON events(timestamp);",
        )
        .map_err(EventListenerError::Database)?;

        Ok(())
    }

    pub fn query_events(
        &self,
        source: Option<String>,
        event_type: Option<String>,
        since: Option<String>,
        limit: Option<usize>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ActivityEvent>, EventListenerError>> + Send + '_>>
    {
        Box::pin(async move {
            let conn = self.conn.clone();

            task::spawn_blocking(move || {
                // We need to block on the async mutex from a sync context inside
                // spawn_blocking. Instead, we'll build the connection Arc and
                // acquire it via tokio's blocking helpers. However, since we're
                // already in spawn_blocking, we can use try_lock or block on it.
                // The safest approach: use a std Mutex instead. But we already
                // have a tokio Mutex. We'll use Handle::current().block_on.
                let rt = tokio::runtime::Handle::current();
                let conn = rt.block_on(conn.lock());

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
                    params.push(Box::new(l as i64));
                }

                let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                    params.iter().map(|p| p.as_ref()).collect();

                let mut stmt = conn.prepare(&sql).map_err(EventListenerError::Database)?;
                let rows = stmt
                    .query_map(param_refs.as_slice(), |row| {
                        Ok(ActivityEvent {
                            id: row.get(0)?,
                            timestamp: row.get(1)?,
                            source: row.get(2)?,
                            event_type: row.get(3)?,
                            payload: row.get(4)?,
                        })
                    })
                    .map_err(EventListenerError::Database)?;

                let mut events = Vec::new();
                for row in rows {
                    events.push(row.map_err(EventListenerError::Database)?);
                }

                Ok(events)
            })
            .await
            .unwrap_or_else(|e| Err(EventListenerError::Other(format!("join error: {e}"))))
        })
    }
}

// ---------------------------------------------------------------------------
// EventListener impl
// ---------------------------------------------------------------------------

#[async_trait]
impl EventListener for SqliteListener {
    async fn on_event(
        &self,
        source: &EventSource,
        event: &Event,
    ) -> Result<(), EventListenerError> {
        let payload = serde_json::to_string(event)?;
        let event_type = event.event_type().to_string();
        let source_str = source.as_str().to_string();
        let conn = self.conn.clone();

        task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let conn = rt.block_on(conn.lock());

            conn.execute(
                "INSERT INTO events (source, event_type, payload) VALUES (?1, ?2, ?3)",
                rusqlite::params![source_str, event_type, payload],
            )
            .map_err(EventListenerError::Database)?;

            Ok(())
        })
        .await
        .unwrap_or_else(|e| Err(EventListenerError::Other(format!("join error: {e}"))))
    }

    fn name(&self) -> &str {
        "sqlite"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SearchEvent;

    fn make_listener() -> SqliteListener {
        let config = SqliteListenerConfig {
            filename: ":memory:".into(),
        };
        SqliteListener::new(&config).unwrap()
    }

    fn make_search_event() -> Event {
        Event::Search(SearchEvent {
            query: "navigation".into(),
            search_type: "hybrid".into(),
            result_count: 7,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        })
    }

    #[tokio::test]
    async fn on_event_stores_and_queries_back() {
        let listener = make_listener();

        listener
            .on_event(&EventSource::Web, &make_search_event())
            .await
            .unwrap();

        let events = listener.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, "web");
        assert_eq!(events[0].event_type, "search");
        assert!(events[0].payload.is_some());

        let payload: serde_json::Value =
            serde_json::from_str(events[0].payload.as_ref().unwrap()).unwrap();
        assert_eq!(payload["query"], "navigation");
    }

    #[tokio::test]
    async fn query_with_filters() {
        let listener = make_listener();

        listener
            .on_event(&EventSource::Web, &make_search_event())
            .await
            .unwrap();
        listener
            .on_event(
                &EventSource::Mcp,
                &Event::ListLeagues(crate::ListLeaguesEvent { result_count: 5 }),
            )
            .await
            .unwrap();

        // Filter by source
        let web_events = listener
            .query_events(Some("web".into()), None, None, None)
            .await
            .unwrap();
        assert_eq!(web_events.len(), 1);
        assert_eq!(web_events[0].source, "web");

        // Filter by event_type
        let league_events = listener
            .query_events(None, Some("list_leagues".into()), None, None)
            .await
            .unwrap();
        assert_eq!(league_events.len(), 1);
        assert_eq!(league_events[0].event_type, "list_leagues");

        // Filter by limit
        let limited = listener
            .query_events(None, None, None, Some(1))
            .await
            .unwrap();
        assert_eq!(limited.len(), 1);
    }

    #[tokio::test]
    async fn ordering_is_desc_by_timestamp() {
        let listener = make_listener();

        // Insert two events
        listener
            .on_event(&EventSource::Web, &make_search_event())
            .await
            .unwrap();
        listener
            .on_event(
                &EventSource::Mcp,
                &Event::ListLeagues(crate::ListLeaguesEvent { result_count: 5 }),
            )
            .await
            .unwrap();

        let events = listener.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 2);
        // Most recent first (DESC) — the second insert should be first
        assert!(events[0].id > events[1].id);
    }
}
