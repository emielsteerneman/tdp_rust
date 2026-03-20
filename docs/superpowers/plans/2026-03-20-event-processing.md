# Event Processing System Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single-backend `ActivityClient` system with a fan-out `EventDispatcher` that delivers typed events to multiple listeners (SQLite + Telegram).

**Architecture:** New `event_processing` crate with an `Event` enum, `EventListener` trait, and `EventDispatcher`. Handlers create typed enum variants, the dispatcher fans out to registered listeners via `tokio::spawn`. SQLite and Telegram are both `EventListener` implementations. The old `data_access::activity` module and `api::activity` module are removed entirely.

**Tech Stack:** Rust, tokio, serde/serde_json, rusqlite, reqwest, async-trait

**Spec:** `docs/superpowers/specs/2026-03-19-event-processing-design.md`

---

## File Map

### New files (event_processing crate)
- `event_processing/Cargo.toml` — crate manifest
- `event_processing/src/lib.rs` — Event enum, EventSource, EventListener trait, error types, re-exports
- `event_processing/src/dispatcher.rs` — EventDispatcher struct
- `event_processing/src/listeners/mod.rs` — module declarations
- `event_processing/src/listeners/sqlite.rs` — SqliteListener + SqliteListenerConfig
- `event_processing/src/listeners/telegram.rs` — TelegramListener + TelegramConfig

### Modified files
- `Cargo.toml` (workspace root) — add `event_processing` to members
- `configuration/Cargo.toml` — add `event_processing` dep, remove `data_access` if no longer needed (check)
- `configuration/src/appconfig.rs` — add `EventProcessingConfig` to `AppConfig`
- `configuration/src/helpers.rs` — replace `load_activity_client` with `build_event_dispatcher`
- `api/Cargo.toml` — add `event_processing` dep
- `api/src/lib.rs` — remove `pub mod activity`
- `api/src/search.rs` — use dispatcher + Event enum
- `api/src/list_leagues.rs` — use dispatcher + Event enum
- `api/src/list_years.rs` — use dispatcher + Event enum
- `api/src/list_teams.rs` — use dispatcher + Event enum
- `api/src/list_papers.rs` — use dispatcher + Event enum
- `api/src/get_abstract.rs` — use dispatcher + Event enum
- `api/src/get_table_of_contents.rs` — use dispatcher + Event enum
- `api/src/get_section.rs` — use dispatcher + Event enum
- `api/src/get_paragraph.rs` — use dispatcher + Event enum
- `api/src/get_table.rs` — use dispatcher + Event enum
- `api/src/get_image.rs` — use dispatcher + Event enum
- `api/src/get_tdp_contents.rs` — use dispatcher + Event enum
- `mcp/Cargo.toml` — add `event_processing` dep
- `mcp/src/state.rs` — replace `activity_client` with `dispatcher`
- `mcp/src/server.rs` — use `EventSource` from event_processing, pass dispatcher
- `mcp/src/main.rs` — use `build_event_dispatcher`
- `web/Cargo.toml` — add `event_processing` dep
- `web/src/state.rs` — replace `activity_client` with `dispatcher`
- `web/src/routes/search.rs` — pass dispatcher instead of activity_client
- `web/src/routes/papers.rs` — use dispatcher + Event enum for paper_open
- `web/src/middleware.rs` — use dispatcher + Event enum for http_request
- `web/src/main.rs` — use `build_event_dispatcher`
- `tools/Cargo.toml` — add `event_processing` dep
- `tools/src/bin/activity.rs` — use SqliteListener directly instead of ActivityClient
- `tools/src/bin/search_by_sentence.rs` — use EventDispatcher instead of activity_client

### Deleted files
- `data_access/src/activity/mod.rs`
- `data_access/src/activity/sqlite_client.rs`

### Modified (cleanup)
- `data_access/src/lib.rs` — remove `pub mod activity`
- `data_access/src/config.rs` — remove `ActivityConfig`
- `data_access/Cargo.toml` — remove `mockall` if no other mocks remain (check)

### Config files
- `config.toml` — move `[data_access.activity.sqlite]` to `[event_processing.activity.sqlite]`, add `[event_processing.telegram]`
- `config.docker.toml` — same changes
- `CLAUDE.md` — update example config

---

## Chunk 1: Core event_processing crate

### Task 1: Create event_processing crate with Event enum and core types

**Files:**
- Create: `event_processing/Cargo.toml`
- Create: `event_processing/src/lib.rs`
- Modify: `Cargo.toml` (workspace root, line 3-11)

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "event_processing"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.48", features = ["macros", "rt-multi-thread"] }
tracing = { workspace = true }
async-trait = "0.1.89"
rusqlite = { version = "0.38.0", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json"] }
thiserror = "2.0"
```

- [ ] **Step 2: Create lib.rs with Event enum, EventSource, EventListener trait, error type**

```rust
pub mod dispatcher;
pub mod listeners;

use async_trait::async_trait;
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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Search(SearchEvent),
    ListLeagues(ListLeaguesEvent),
    ListYears(ListYearsEvent),
    ListTeams(ListTeamsEvent),
    ListPapers(ListPapersEvent),
    GetAbstract(GetAbstractEvent),
    GetTableOfContents(GetTableOfContentsEvent),
    GetSection(GetSectionEvent),
    GetParagraph(GetParagraphEvent),
    GetTable(GetTableEvent),
    GetImage(GetImageEvent),
    GetTdpContents(GetTdpContentsEvent),
    HttpRequest(HttpRequestEvent),
    PaperOpen(PaperOpenEvent),
}

impl Event {
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::Search(_) => "search",
            Event::ListLeagues(_) => "list_leagues",
            Event::ListYears(_) => "list_years",
            Event::ListTeams(_) => "list_teams",
            Event::ListPapers(_) => "list_papers",
            Event::GetAbstract(_) => "get_abstract",
            Event::GetTableOfContents(_) => "get_table_of_contents",
            Event::GetSection(_) => "get_section",
            Event::GetParagraph(_) => "get_paragraph",
            Event::GetTable(_) => "get_table",
            Event::GetImage(_) => "get_image",
            Event::GetTdpContents(_) => "get_tdp_contents",
            Event::HttpRequest(_) => "http_request",
            Event::PaperOpen(_) => "paper_open",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchEvent {
    pub query: String,
    pub search_type: String,
    pub result_count: usize,
    pub league_filter: Option<String>,
    pub year_filter: Option<String>,
    pub team_filter: Option<String>,
    pub content_type_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListLeaguesEvent {
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListYearsEvent {
    pub league: Option<String>,
    pub team: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListTeamsEvent {
    pub hint: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListPapersEvent {
    pub league: Option<String>,
    pub year: Option<String>,
    pub team: Option<String>,
    pub result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetAbstractEvent {
    pub paper: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTableOfContentsEvent {
    pub paper: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetSectionEvent {
    pub paper: String,
    pub content_seq: u32,
    pub include_children: bool,
    pub items_returned: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetParagraphEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTableEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetImageEvent {
    pub paper: String,
    pub content_seq: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTdpContentsEvent {
    pub league: String,
    pub year: String,
    pub team: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpRequestEvent {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub ip: Option<String>,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaperOpenEvent {
    pub paper_id: String,
    pub referrer: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum EventListenerError {
    #[error("Listener error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError>;
    fn name(&self) -> &str;
}
```

- [ ] **Step 3: Add event_processing to workspace members**

In `Cargo.toml` (workspace root), add `"event_processing"` to the members list.

- [ ] **Step 4: Create stub listener module files**

Create `event_processing/src/listeners/mod.rs`:
```rust
pub mod sqlite;
pub mod telegram;
```

Create empty stub files so the crate compiles:
- `event_processing/src/dispatcher.rs` — empty module (filled in Task 2)
- `event_processing/src/listeners/sqlite.rs` — empty module (filled in Task 3)
- `event_processing/src/listeners/telegram.rs` — empty module (filled in Task 4)

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p event_processing`
Expected: compiles with no errors

- [ ] **Step 6: Write tests for Event enum**

Add to the bottom of `event_processing/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_strings() {
        assert_eq!(Event::Search(SearchEvent {
            query: "test".into(),
            search_type: "hybrid".into(),
            result_count: 0,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        }).event_type(), "search");

        assert_eq!(Event::ListLeagues(ListLeaguesEvent { result_count: 0 }).event_type(), "list_leagues");
        assert_eq!(Event::PaperOpen(PaperOpenEvent { paper_id: "x".into(), referrer: None }).event_type(), "paper_open");
    }

    #[test]
    fn test_event_source_as_str() {
        assert_eq!(EventSource::Web.as_str(), "web");
        assert_eq!(EventSource::Mcp.as_str(), "mcp");
    }

    #[test]
    fn test_event_serializes_to_json() {
        let event = Event::Search(SearchEvent {
            query: "trajectory".into(),
            search_type: "hybrid".into(),
            result_count: 5,
            league_filter: Some("Soccer SmallSize".into()),
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "search");
        assert_eq!(json["query"], "trajectory");
        assert_eq!(json["result_count"], 5);
    }
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p event_processing`
Expected: all 3 tests pass

- [ ] **Step 8: Commit**

```bash
git add event_processing/ Cargo.toml
git commit -m "feat: create event_processing crate with Event enum and core types"
```

### Task 2: Implement EventDispatcher

**Files:**
- Modify: `event_processing/src/dispatcher.rs`

- [ ] **Step 1: Write tests for EventDispatcher**

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{Event, EventListener, EventListenerError, EventSource, SearchEvent};

pub struct EventDispatcher {
    listeners: Vec<Arc<dyn EventListener>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn register(&mut self, listener: Arc<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub fn dispatch(&self, source: EventSource, event: Event) {
        let event = Arc::new(event);
        let source = Arc::new(source);
        for listener in &self.listeners {
            let listener = listener.clone();
            let event = event.clone();
            let source = source.clone();
            tokio::spawn(async move {
                if let Err(e) = listener.on_event(&source, &event).await {
                    tracing::warn!("Event listener '{}' failed: {}", listener.name(), e);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct RecordingListener {
        events: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl RecordingListener {
        fn new() -> (Self, Arc<Mutex<Vec<(String, String)>>>) {
            let events = Arc::new(Mutex::new(Vec::new()));
            (Self { events: events.clone() }, events)
        }
    }

    #[async_trait]
    impl EventListener for RecordingListener {
        async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError> {
            self.events.lock().await.push((
                source.as_str().to_string(),
                event.event_type().to_string(),
            ));
            Ok(())
        }
        fn name(&self) -> &str { "recording" }
    }

    struct FailingListener;

    #[async_trait]
    impl EventListener for FailingListener {
        async fn on_event(&self, _: &EventSource, _: &Event) -> Result<(), EventListenerError> {
            Err(EventListenerError::Internal("boom".into()))
        }
        fn name(&self) -> &str { "failing" }
    }

    #[tokio::test]
    async fn test_dispatch_to_single_listener() {
        let (listener, events) = RecordingListener::new();
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Arc::new(listener));

        dispatcher.dispatch(EventSource::Web, Event::Search(SearchEvent {
            query: "test".into(),
            search_type: "hybrid".into(),
            result_count: 3,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        }));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let recorded = events.lock().await;
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0], ("web".to_string(), "search".to_string()));
    }

    #[tokio::test]
    async fn test_dispatch_to_multiple_listeners() {
        let (listener1, events1) = RecordingListener::new();
        let (listener2, events2) = RecordingListener::new();
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Arc::new(listener1));
        dispatcher.register(Arc::new(listener2));

        dispatcher.dispatch(EventSource::Mcp, Event::ListLeagues(crate::ListLeaguesEvent { result_count: 5 }));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(events1.lock().await.len(), 1);
        assert_eq!(events2.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_empty_dispatcher_is_noop() {
        let dispatcher = EventDispatcher::new();
        // Should not panic
        dispatcher.dispatch(EventSource::Web, Event::ListLeagues(crate::ListLeaguesEvent { result_count: 0 }));
    }

    #[tokio::test]
    async fn test_failing_listener_does_not_affect_others() {
        let (listener, events) = RecordingListener::new();
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Arc::new(FailingListener));
        dispatcher.register(Arc::new(listener));

        dispatcher.dispatch(EventSource::Web, Event::ListLeagues(crate::ListLeaguesEvent { result_count: 0 }));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(events.lock().await.len(), 1);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p event_processing`
Expected: all tests pass (7 total: 3 from Task 1 + 4 new)

- [ ] **Step 3: Commit**

```bash
git add event_processing/src/dispatcher.rs
git commit -m "feat: implement EventDispatcher with fan-out to listeners"
```

### Task 3: Implement SqliteListener

**Files:**
- Modify: `event_processing/src/listeners/sqlite.rs`

- [ ] **Step 1: Write SqliteListener implementation and tests**

Port from `data_access/src/activity/sqlite_client.rs`, adapting to the `EventListener` trait. Keep `query_events` as a concrete method.

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rusqlite::Connection;
use serde::Deserialize;

use crate::{Event, EventListener, EventListenerError, EventSource};

#[derive(Debug, Deserialize, Clone)]
pub struct SqliteListenerConfig {
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct ActivityEvent {
    pub id: i64,
    pub timestamp: String,
    pub source: String,
    pub event_type: String,
    pub payload: Option<String>,
}

pub struct SqliteListener {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteListener {
    pub fn new(config: SqliteListenerConfig) -> Self {
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

    pub fn query_events(
        &self,
        source: Option<String>,
        event_type: Option<String>,
        since: Option<String>,
        limit: Option<u32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ActivityEvent>, EventListenerError>> + Send + '_>> {
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
                    .map_err(|e| EventListenerError::Internal(e.to_string()))?;

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
                    .map_err(|e| EventListenerError::Internal(e.to_string()))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| EventListenerError::Internal(e.to_string()))?;

                Ok(events)
            })
            .await
            .map_err(|e| EventListenerError::Internal(e.to_string()))?
        })
    }
}

#[async_trait]
impl EventListener for SqliteListener {
    fn name(&self) -> &str {
        "sqlite"
    }

    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError> {
        let conn = self.conn.clone();
        let source_str = source.as_str().to_string();
        let event_type = event.event_type().to_string();
        let payload = serde_json::to_string(event)
            .map_err(|e| EventListenerError::Internal(e.to_string()))?;

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            conn.execute(
                "INSERT INTO events (source, event_type, payload) VALUES (?1, ?2, ?3)",
                rusqlite::params![source_str, event_type, payload],
            )
            .map_err(|e| EventListenerError::Internal(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| EventListenerError::Internal(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ListTeamsEvent, SearchEvent};

    fn temp_listener() -> SqliteListener {
        SqliteListener::new(SqliteListenerConfig {
            filename: ":memory:".to_string(),
        })
    }

    #[tokio::test]
    async fn test_on_event_stores_event() {
        let listener = temp_listener();
        let event = Event::Search(SearchEvent {
            query: "trajectory".into(),
            search_type: "hybrid".into(),
            result_count: 5,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });

        listener.on_event(&EventSource::Web, &event).await.unwrap();

        let events = listener.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, "web");
        assert_eq!(events[0].event_type, "search");
        assert!(events[0].payload.as_ref().unwrap().contains("trajectory"));
    }

    #[tokio::test]
    async fn test_query_events_with_filters() {
        let listener = temp_listener();

        listener.on_event(&EventSource::Web, &Event::Search(SearchEvent {
            query: "test".into(),
            search_type: "hybrid".into(),
            result_count: 0,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        })).await.unwrap();

        listener.on_event(&EventSource::Mcp, &Event::ListTeams(ListTeamsEvent {
            hint: Some("tiger".into()),
            result_count: 3,
        })).await.unwrap();

        let all = listener.query_events(None, None, None, None).await.unwrap();
        assert_eq!(all.len(), 2);

        let web_only = listener.query_events(Some("web".into()), None, None, None).await.unwrap();
        assert_eq!(web_only.len(), 1);

        let searches = listener.query_events(None, Some("search".into()), None, None).await.unwrap();
        assert_eq!(searches.len(), 1);

        let limited = listener.query_events(None, None, None, Some(1)).await.unwrap();
        assert_eq!(limited.len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_events_ordering() {
        let listener = temp_listener();

        listener.on_event(&EventSource::Web, &Event::Search(SearchEvent {
            query: "first".into(),
            search_type: "hybrid".into(),
            result_count: 0,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        })).await.unwrap();

        listener.on_event(&EventSource::Web, &Event::Search(SearchEvent {
            query: "second".into(),
            search_type: "hybrid".into(),
            result_count: 0,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        })).await.unwrap();

        let events = listener.query_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 2);
        // DESC order
        assert!(events[0].timestamp >= events[1].timestamp);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p event_processing`
Expected: all tests pass (7 prior + 3 new = 10)

- [ ] **Step 3: Commit**

```bash
git add event_processing/src/listeners/sqlite.rs
git commit -m "feat: implement SqliteListener for event storage"
```

### Task 4: Implement TelegramListener

**Files:**
- Modify: `event_processing/src/listeners/telegram.rs`

- [ ] **Step 1: Write TelegramListener implementation and tests**

```rust
use async_trait::async_trait;
use serde::Deserialize;

use crate::{Event, EventListener, EventListenerError, EventSource};

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

pub struct TelegramListener {
    client: reqwest::Client,
    bot_token: String,
    chat_id: String,
}

impl TelegramListener {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token: config.bot_token,
            chat_id: config.chat_id,
        }
    }

    fn format_message(&self, source: &EventSource, event: &Event) -> Option<String> {
        let src = source.as_str();
        match event {
            Event::Search(e) => {
                let mut msg = format!("[{}] Search: '{}' ({}, {} results)", src, e.query, e.search_type, e.result_count);
                if let Some(ref f) = e.league_filter {
                    msg.push_str(&format!("\n  league: {}", f));
                }
                if let Some(ref f) = e.year_filter {
                    msg.push_str(&format!("\n  year: {}", f));
                }
                if let Some(ref f) = e.team_filter {
                    msg.push_str(&format!("\n  team: {}", f));
                }
                Some(msg)
            }
            Event::GetAbstract(e) => Some(format!("[{}] Read abstract: {}", src, e.paper)),
            Event::GetTableOfContents(e) => Some(format!("[{}] Read TOC: {}", src, e.paper)),
            Event::GetSection(e) => Some(format!("[{}] Read section: {} seq={} ({} items)", src, e.paper, e.content_seq, e.items_returned)),
            Event::GetParagraph(e) => Some(format!("[{}] Read paragraph: {} seq={}", src, e.paper, e.content_seq)),
            Event::GetTable(e) => Some(format!("[{}] Read table: {} seq={}", src, e.paper, e.content_seq)),
            Event::GetImage(e) => Some(format!("[{}] Read image: {} seq={}", src, e.paper, e.content_seq)),
            Event::GetTdpContents(e) => Some(format!("[{}] Read full TDP: {} {} {}", src, e.league, e.year, e.team)),
            Event::PaperOpen(e) => {
                let referrer = e.referrer.as_deref().unwrap_or("direct");
                Some(format!("[{}] Paper opened: {} (from: {})", src, e.paper_id, referrer))
            }
            // Skip noisy/low-value events
            Event::ListLeagues(_) | Event::ListYears(_) | Event::ListTeams(_)
            | Event::ListPapers(_) | Event::HttpRequest(_) => None,
        }
    }
}

#[async_trait]
impl EventListener for TelegramListener {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError> {
        let Some(message) = self.format_message(source, event) else {
            return Ok(());
        };

        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": self.chat_id,
                "text": message,
            }))
            .send()
            .await
            .map_err(|e| EventListenerError::Internal(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EventListenerError::Internal(
                format!("Telegram API error {}: {}", status, body),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn test_listener() -> TelegramListener {
        TelegramListener::new(TelegramConfig {
            bot_token: "test_token".into(),
            chat_id: "test_chat".into(),
        })
    }

    #[test]
    fn test_format_search_event() {
        let listener = test_listener();
        let event = Event::Search(SearchEvent {
            query: "trajectory planning".into(),
            search_type: "hybrid".into(),
            result_count: 7,
            league_filter: Some("Soccer SmallSize".into()),
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });
        let msg = listener.format_message(&EventSource::Web, &event).unwrap();
        assert!(msg.contains("trajectory planning"));
        assert!(msg.contains("7 results"));
        assert!(msg.contains("Soccer SmallSize"));
        assert!(msg.contains("[web]"));
    }

    #[test]
    fn test_format_search_event_mcp() {
        let listener = test_listener();
        let event = Event::Search(SearchEvent {
            query: "ball detection".into(),
            search_type: "dense".into(),
            result_count: 3,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        });
        let msg = listener.format_message(&EventSource::Mcp, &event).unwrap();
        assert!(msg.contains("[mcp]"));
        assert!(msg.contains("ball detection"));
    }

    #[test]
    fn test_format_paper_open_event() {
        let listener = test_listener();
        let event = Event::PaperOpen(PaperOpenEvent {
            paper_id: "soccer_smallsize__2024__TIGERs__0".into(),
            referrer: Some("https://google.com".into()),
        });
        let msg = listener.format_message(&EventSource::Web, &event).unwrap();
        assert!(msg.contains("TIGERs"));
        assert!(msg.contains("google.com"));
    }

    #[test]
    fn test_format_get_abstract_event() {
        let listener = test_listener();
        let event = Event::GetAbstract(GetAbstractEvent {
            paper: "soccer_smallsize__2024__RoboTeam_Twente__0".into(),
        });
        let msg = listener.format_message(&EventSource::Mcp, &event).unwrap();
        assert!(msg.contains("abstract"));
        assert!(msg.contains("RoboTeam_Twente"));
    }

    #[test]
    fn test_skips_list_events() {
        let listener = test_listener();
        assert!(listener.format_message(&EventSource::Web, &Event::ListLeagues(ListLeaguesEvent { result_count: 5 })).is_none());
        assert!(listener.format_message(&EventSource::Web, &Event::ListTeams(ListTeamsEvent { hint: None, result_count: 0 })).is_none());
        assert!(listener.format_message(&EventSource::Web, &Event::HttpRequest(HttpRequestEvent {
            method: "GET".into(), path: "/".into(), status: 200, duration_ms: 10, ip: None, user_agent: "test".into(),
        })).is_none());
    }

    #[test]
    fn test_format_get_tdp_contents() {
        let listener = test_listener();
        let event = Event::GetTdpContents(GetTdpContentsEvent {
            league: "Soccer SmallSize".into(),
            year: "2024".into(),
            team: "TIGERs Mannheim".into(),
        });
        let msg = listener.format_message(&EventSource::Mcp, &event).unwrap();
        assert!(msg.contains("TIGERs Mannheim"));
        assert!(msg.contains("2024"));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p event_processing`
Expected: all tests pass (10 prior + 6 new = 16)

- [ ] **Step 3: Commit**

```bash
git add event_processing/src/listeners/telegram.rs
git commit -m "feat: implement TelegramListener with message formatting"
```

---

## Chunk 2: Configuration and wiring

### Task 5: Add event_processing config types and build_event_dispatcher

**Files:**
- Modify: `configuration/Cargo.toml` — add `event_processing` dep
- Modify: `configuration/src/appconfig.rs` — add `EventProcessingConfig`
- Modify: `configuration/src/helpers.rs` — add `build_event_dispatcher`, keep `load_activity_client` temporarily

- [ ] **Step 1: Add event_processing dependency to configuration/Cargo.toml**

Add under `[dependencies]`:
```toml
event_processing = { path = "../event_processing" }
```

- [ ] **Step 2: Add EventProcessingConfig to appconfig.rs**

In `configuration/src/appconfig.rs`, add to the `AppConfig` struct:

```rust
pub event_processing: Option<EventProcessingConfig>,
```

Add the config struct (can go in the same file or a new module — keep it in appconfig.rs for now since it's small):

```rust
use event_processing::listeners::sqlite::SqliteListenerConfig;
use event_processing::listeners::telegram::TelegramConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct EventProcessingConfig {
    pub activity: Option<ActivityListenerConfig>,
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActivityListenerConfig {
    pub sqlite: Option<SqliteListenerConfig>,
}
```

- [ ] **Step 3: Add build_event_dispatcher to helpers.rs**

```rust
use event_processing::dispatcher::EventDispatcher;
use event_processing::listeners::sqlite::SqliteListener;
use event_processing::listeners::telegram::TelegramListener;

pub fn build_event_dispatcher(config: &AppConfig) -> Arc<EventDispatcher> {
    let mut dispatcher = EventDispatcher::new();

    if let Some(ref ep_config) = config.event_processing {
        if let Some(ref activity_config) = ep_config.activity {
            if let Some(ref sqlite_cfg) = activity_config.sqlite {
                info!("Registering SQLite event listener: {}", sqlite_cfg.filename);
                dispatcher.register(Arc::new(SqliteListener::new(sqlite_cfg.clone())));
            }
        }

        if let Some(ref telegram_cfg) = ep_config.telegram {
            info!("Registering Telegram event listener");
            dispatcher.register(Arc::new(TelegramListener::new(telegram_cfg.clone())));
        }
    }

    Arc::new(dispatcher)
}
```

- [ ] **Step 4: Update test configs in appconfig.rs tests**

The existing `test_simple_config` and `test_full_config` tests need either:
- No event_processing section (since it's `Option`), or
- An `[event_processing]` section added.

Since it's `Option`, existing tests should still pass without changes. Verify.

- [ ] **Step 5: Verify it compiles and tests pass**

Run: `cargo test -p configuration`
Expected: all existing tests pass

- [ ] **Step 6: Commit**

```bash
git add configuration/
git commit -m "feat: add EventProcessingConfig and build_event_dispatcher"
```

### Task 6: Update api crate to use event_processing

**Files:**
- Modify: `api/Cargo.toml` — add `event_processing` dep
- Modify: `api/src/lib.rs` — remove `pub mod activity`
- Modify: all 12 handler files in `api/src/`

- [ ] **Step 1: Add event_processing dependency to api/Cargo.toml**

Add under `[dependencies]`:
```toml
event_processing = { path = "../event_processing" }
```

- [ ] **Step 2: Update api/src/lib.rs**

Remove the line `pub mod activity;`.

- [ ] **Step 3: Update each handler**

For each handler file, apply this pattern:

**Replace imports:**
```rust
// Remove:
use data_access::activity::ActivityClient;
use crate::activity::{EventSource, log_activity};

// Add:
use event_processing::{Event, EventSource};
use event_processing::dispatcher::EventDispatcher;
```

**Replace function signatures:**
```rust
// Remove:
activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
source: EventSource,

// Replace with:
dispatcher: &EventDispatcher,
source: EventSource,
```

**Replace log_activity calls with dispatcher.dispatch calls.**

Here are the specific changes per file:

**`api/src/list_leagues.rs`:**
```rust
use std::sync::Arc;
use data_access::metadata::MetadataClient;
use data_structures::file::League;
use event_processing::{Event, EventSource, ListLeaguesEvent};
use event_processing::dispatcher::EventDispatcher;
use crate::error::ApiError;

pub async fn list_leagues(
    metadata_client: Arc<dyn MetadataClient>,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<League>, ApiError> {
    let leagues = metadata_client
        .load_leagues()
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;

    dispatcher.dispatch(source, Event::ListLeagues(ListLeaguesEvent {
        result_count: leagues.len(),
    }));

    Ok(leagues)
}
```

**`api/src/list_years.rs`** — same pattern, `Event::ListYears(ListYearsEvent { league: filter.league, team: filter.team, result_count: result.len() })`

**`api/src/list_teams.rs`** — same pattern, `Event::ListTeams(ListTeamsEvent { hint: args.hint.clone(), result_count: teams.len() })`

**`api/src/list_papers.rs`** — same pattern, `Event::ListPapers(ListPapersEvent { league: filter.league, year: filter.year, team: filter.team, result_count: result.len() })`

**`api/src/get_abstract.rs`** — same pattern, `Event::GetAbstract(GetAbstractEvent { paper: args.paper.clone() })`

**`api/src/get_table_of_contents.rs`** — same pattern, `Event::GetTableOfContents(GetTableOfContentsEvent { paper: args.paper.clone() })`

**`api/src/get_section.rs`** — same pattern, `Event::GetSection(GetSectionEvent { paper: args.paper.clone(), content_seq: args.content_seq, include_children, items_returned: items.len() })`

**`api/src/get_paragraph.rs`** — same pattern, `Event::GetParagraph(GetParagraphEvent { paper: args.paper.clone(), content_seq: args.content_seq })`

**`api/src/get_table.rs`** — same pattern, `Event::GetTable(GetTableEvent { paper: args.paper.clone(), content_seq: args.content_seq })`

**`api/src/get_image.rs`** — same pattern, `Event::GetImage(GetImageEvent { paper: args.paper.clone(), content_seq: args.content_seq })`

**`api/src/get_tdp_contents.rs`** — same pattern, `Event::GetTdpContents(GetTdpContentsEvent { league: args.league.clone(), year: args.year.to_string(), team: args.team.clone() })`

**`api/src/search.rs`:**
```rust
dispatcher.dispatch(source, Event::Search(SearchEvent {
    query: args.query.clone(),
    search_type: search_type_str,
    result_count: search_result.chunks.len(),
    league_filter: args.league_filter.clone(),
    year_filter: args.year_filter.clone(),
    team_filter: args.team_filter.clone(),
    content_type_filter: args.content_type_filter.clone(),
}));
```

- [ ] **Step 4: Update tests in handler files**

All handler tests currently pass `None` for `activity_client` and `EventSource::Web` or `EventSource::Dev`. Replace with a no-listener dispatcher:

```rust
// In test imports:
use event_processing::EventSource;
use event_processing::dispatcher::EventDispatcher;

// Replace:
//   None, EventSource::Web
// With:
//   &EventDispatcher::new(), EventSource::Web
```

Note: tests that used `EventSource::Dev` should use `EventSource::Web` now (Dev is removed). The empty dispatcher serves the same purpose.

- [ ] **Step 5: Delete api/src/activity.rs**

Remove the file entirely.

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p api`
Expected: compiles with no errors

- [ ] **Step 7: Run api tests**

Run: `cargo test -p api`
Expected: all tests pass

- [ ] **Step 8: Commit**

```bash
git add api/
git commit -m "refactor: update api handlers to use EventDispatcher"
```

---

## Chunk 3: Server wiring and cleanup

### Task 7: Update mcp and web servers

**Files:**
- Modify: `mcp/Cargo.toml`, `mcp/src/state.rs`, `mcp/src/server.rs`, `mcp/src/main.rs`
- Modify: `web/Cargo.toml`, `web/src/state.rs`, `web/src/routes/search.rs`, `web/src/routes/papers.rs`, `web/src/middleware.rs`, `web/src/main.rs`
- Modify: all other web route files that pass `activity_client`

- [ ] **Step 1: Add event_processing dep to mcp/Cargo.toml and web/Cargo.toml**

Add to both:
```toml
event_processing = { path = "../event_processing" }
```

- [ ] **Step 2: Update mcp/src/state.rs**

```rust
use data_access::metadata::MetadataClient;
use data_processing::search::Searcher;
use event_processing::dispatcher::EventDispatcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
        }
    }
}
```

- [ ] **Step 3: Update mcp/src/server.rs**

Replace all `self.state.activity_client.clone(), api::activity::EventSource::Mcp` with `&self.state.dispatcher, event_processing::EventSource::Mcp`.

For each tool method, change the handler call. Example for search:
```rust
match search::search(&self.state.searcher, args, &self.state.dispatcher, event_processing::EventSource::Mcp).await {
```

Same pattern for `list_teams`, `list_leagues`, `list_years`, `list_papers`, `get_tdp_contents`, `get_table_of_contents`, `get_section`, `get_abstract`.

- [ ] **Step 4: Update mcp/src/main.rs**

Replace:
```rust
let activity_client = configuration::helpers::load_activity_client(&config);
```
With:
```rust
let dispatcher = configuration::helpers::build_event_dispatcher(&config);
```

And update the AppState construction:
```rust
let state = AppState::new(metadata_client.clone(), Arc::new(searcher), dispatcher);
```

- [ ] **Step 5: Update web/src/state.rs**

```rust
use data_access::metadata::MetadataClient;
use data_processing::search::Searcher;
use event_processing::dispatcher::EventDispatcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
    pub tdps_markdown_root: String,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        tdps_markdown_root: String,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            tdps_markdown_root,
        }
    }
}
```

- [ ] **Step 6: Update web route handlers**

**`web/src/routes/search.rs`:**
```rust
let result = api::search::search(
    &state.searcher,
    args,
    &state.dispatcher,
    event_processing::EventSource::Web,
)
```

**`web/src/routes/papers.rs` (list_papers_handler):**
```rust
let papers = api::list_papers::list_papers(
    state.metadata_client.clone(),
    api::paper_filter::PaperFilter::default(),
    &state.dispatcher,
    event_processing::EventSource::Web,
)
```

**`web/src/routes/papers.rs` (get_paper_handler):**
Replace the `api::activity::log_activity(...)` block with:
```rust
use event_processing::{Event, PaperOpenEvent};

state.dispatcher.dispatch(
    event_processing::EventSource::Web,
    Event::PaperOpen(PaperOpenEvent {
        paper_id: lyti,
        referrer,
    }),
);
```

**All other web route files** (leagues.rs, years.rs, teams.rs, table_of_contents.rs, abstract_text.rs, paragraph.rs, table.rs, image.rs) — same pattern: replace `state.activity_client.clone(), api::activity::EventSource::Web` with `&state.dispatcher, event_processing::EventSource::Web`.

- [ ] **Step 7: Update web/src/middleware.rs**

```rust
use crate::state::AppState;
use axum::body::Body;
use axum::http::{Request, header};
use axum::middleware::Next;
use axum::response::Response;
use event_processing::{Event, HttpRequestEvent};
use std::time::Instant;

pub async fn activity_logging(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let start = Instant::now();
    let response = next.run(request).await;
    let duration_ms = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();

    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        Event::HttpRequest(HttpRequestEvent {
            method,
            path,
            status,
            duration_ms,
            ip,
            user_agent,
        }),
    );

    response
}
```

- [ ] **Step 8: Update web/src/main.rs**

Replace:
```rust
let activity_client = configuration::helpers::load_activity_client(&config);
```
With:
```rust
let dispatcher = configuration::helpers::build_event_dispatcher(&config);
```

And update AppState construction:
```rust
let state = AppState::new(
    metadata_client.clone(),
    Arc::new(searcher),
    dispatcher,
    config.data_processing.tdps_markdown_root.clone(),
);
```

- [ ] **Step 9: Verify it compiles**

Run: `cargo check`
Expected: full workspace compiles

- [ ] **Step 10: Commit**

```bash
git add mcp/ web/
git commit -m "refactor: update mcp and web servers to use EventDispatcher"
```

### Task 8: Update tools/activity CLI and remove old code

**Files:**
- Modify: `tools/Cargo.toml` — add `event_processing` dep
- Modify: `tools/src/bin/activity.rs` — use SqliteListener directly
- Delete: `data_access/src/activity/mod.rs`, `data_access/src/activity/sqlite_client.rs`
- Modify: `data_access/src/lib.rs` — remove `pub mod activity`
- Modify: `data_access/src/config.rs` — remove `ActivityConfig`
- Modify: `configuration/src/helpers.rs` — remove `load_activity_client`

- [ ] **Step 1: Add event_processing dep to tools/Cargo.toml**

```toml
event_processing = { path = "../event_processing" }
```

- [ ] **Step 2: Update tools/src/bin/search_by_sentence.rs**

Replace:
```rust
let activity_client = configuration::helpers::load_activity_client(&config);
```
With:
```rust
let dispatcher = configuration::helpers::build_event_dispatcher(&config);
```

And update the search call:
```rust
let results = search(&searcher, search_args, &dispatcher, event_processing::EventSource::Web)
    .await?;
```

Remove `use api::activity::EventSource;` and add `use event_processing::EventSource;` if needed. Use `EventSource::Web` instead of `EventSource::Dev` (Dev no longer exists; for CLI tools, Web or Mcp doesn't matter since no listeners care about the source for search events).

- [ ] **Step 3: Update tools/src/bin/activity.rs**

Replace all `ActivityClient` usage with `SqliteListener` from event_processing:

```rust
use std::collections::HashMap;

use event_processing::listeners::sqlite::{SqliteListener, SqliteListenerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let sqlite_cfg = config
        .event_processing
        .as_ref()
        .and_then(|ep| ep.activity.as_ref())
        .and_then(|a| a.sqlite.as_ref())
        .ok_or_else(|| anyhow::anyhow!("No activity SQLite listener configured in config.toml"))?;

    let listener = SqliteListener::new(sqlite_cfg.clone());

    let args: Vec<String> = std::env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("summary");

    match subcommand {
        "summary" => summary(&listener, args.get(2..).unwrap_or_default()).await?,
        "recent" => recent(&listener, args.get(2..).unwrap_or_default()).await?,
        "agents" => agents(&listener, args.get(2..).unwrap_or_default()).await?,
        _ => {
            eprintln!("Usage: activity <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  summary [--since DATE]   Event counts by type and source");
            eprintln!("  recent  [--limit N]      Most recent events");
            eprintln!("  agents  [--since DATE]   User-agent breakdown (scraper detection)");
        }
    }

    Ok(())
}
```

Update the function signatures from `client: &Arc<dyn ActivityClient + Send + Sync>` to `listener: &SqliteListener`. The `query_events` call stays the same (it's a concrete method on `SqliteListener` with the same signature).

- [ ] **Step 4: Delete old activity module from data_access**

Remove files:
- `data_access/src/activity/mod.rs`
- `data_access/src/activity/sqlite_client.rs`

- [ ] **Step 5: Update data_access/src/lib.rs**

Remove the line `pub mod activity;`.

- [ ] **Step 6: Update data_access/src/config.rs**

Remove `ActivityConfig` and the `use` for `ActivitySqliteConfig`:

```rust
// Remove:
use crate::activity::ActivitySqliteConfig;

// Remove from DataAccessConfig:
pub activity: Option<ActivityConfig>,

// Remove entirely:
#[derive(Debug, Deserialize, Clone)]
pub struct ActivityConfig {
    pub sqlite: Option<ActivitySqliteConfig>,
}
```

- [ ] **Step 7: Remove load_activity_client from configuration/src/helpers.rs**

Delete the entire `load_activity_client` function and its imports (`ActivityClient`, `ActivitySqliteClient`).

- [ ] **Step 8: Check if data_access still needs mockall**

Run: `grep -r "automock\|MockMetadata" data_access/src/`

If `MetadataClient` still uses `#[automock]`, keep mockall. Otherwise remove from `data_access/Cargo.toml`.

- [ ] **Step 9: Verify full workspace compiles and tests pass**

Run: `cargo test`
Expected: all tests pass across all crates

- [ ] **Step 10: Commit**

```bash
git add tools/ data_access/ configuration/
git commit -m "refactor: remove old ActivityClient, use SqliteListener in CLI"
```

### Task 9: Update config files and documentation

**Files:**
- Modify: `config.toml`
- Modify: `config.docker.toml`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update config.toml**

Remove:
```toml
[data_access.activity.sqlite]
filename = "data/activity.db"
```

Add:
```toml
[event_processing.activity.sqlite]
filename = "data/activity.db"

# [event_processing.telegram]
# bot_token = "123456:ABC-DEF..."
# chat_id = "987654321"
```

- [ ] **Step 2: Update config.docker.toml**

Same changes — move activity sqlite config from `data_access` to `event_processing`, add commented-out telegram section.

- [ ] **Step 3: Update CLAUDE.md**

Update the example config.toml snippet to reflect the new structure. Replace the `[data_access.activity.sqlite]` section with:

```toml
[event_processing.activity.sqlite]
filename = "data/activity.db"

# Optional: Telegram notifications
# [event_processing.telegram]
# bot_token = "123456:ABC-DEF..."
# chat_id = "987654321"
```

Also update:
- Architecture description to mention `event_processing` crate
- Remove mention of `ActivityClient` trait
- Update "Adding a New Tool / Endpoint" section to mention dispatcher instead of activity_client

- [ ] **Step 4: Final full test run**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add config.toml config.docker.toml CLAUDE.md
git commit -m "docs: update config files and CLAUDE.md for event_processing"
```

### Task 10: Create feature branch and verify

- [ ] **Step 1: Verify no remaining references to old code**

Run: `cargo test`
Then search for leftover references:
```bash
grep -r "ActivityClient" --include="*.rs" .
grep -r "log_activity" --include="*.rs" .
grep -r "data_access::activity" --include="*.rs" .
grep -r "api::activity" --include="*.rs" .
```

Expected: no matches (except possibly in git history)

- [ ] **Step 2: Verify clean build**

Run: `cargo build`
Expected: builds with no warnings related to our changes
