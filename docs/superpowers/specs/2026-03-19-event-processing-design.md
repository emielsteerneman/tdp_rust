# Event Processing System Design

## Problem

Activity logging is currently tightly coupled to a single SQLite backend via the `ActivityClient` trait. The user wants Telegram notifications when the API is used (to see what users and LLMs search for), and the system should support adding more listeners in the future without changing handler code.

## Goals

- Decouple event emission from event consumption
- Support multiple listeners (SQLite logging, Telegram notifications, future listeners)
- Type-safe events end-to-end (handlers create typed variants, listeners match on them)
- Fire-and-forget semantics (listeners never block API responses)
- Easy to add/remove listeners via config

## Non-Goals

- Event replay or persistence guarantees beyond SQLite
- Backpressure, ordering, or exactly-once delivery
- Batching or digest-style notifications (can be added later per-listener)

## Architecture

### New Crate: `event_processing`

```
event_processing/
  src/
    lib.rs              -- Event enum, EventSource enum, EventListener trait
    dispatcher.rs       -- EventDispatcher
    listeners/
      mod.rs
      sqlite.rs         -- SqliteListener (event storage + query)
      telegram.rs       -- TelegramListener
```

### Core Types

```rust
/// Source of the event (which server interface)
pub enum EventSource {
    Web,
    Mcp,
}

/// All possible events in the system. Type-safe end-to-end.
/// Handlers create variants, listeners pattern-match on them.
#[derive(Debug, Clone, Serialize)]
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
    pub content_seq: Option<u32>,
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
```

### Event Helper Methods

`Event` provides helper methods so listeners don't need to repeat logic:

```rust
impl Event {
    /// Returns the event type as a string (for SQLite storage)
    pub fn event_type(&self) -> &'static str {
        match self {
            Event::Search(_) => "search",
            Event::ListLeagues(_) => "list_leagues",
            // ...
        }
    }
}
```

### EventListener Trait

```rust
/// Implemented by all event consumers
#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError>;
    fn name(&self) -> &str;
}
```

Listeners receive both `source` and `event` as separate arguments (source is not part of the event -- it's delivery context).

### EventDispatcher

```rust
pub struct EventDispatcher {
    listeners: Vec<Arc<dyn EventListener>>,
}

impl EventDispatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, listener: Arc<dyn EventListener>);

    /// Spawn a task per listener (fire-and-forget)
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
```

Key properties:
- `Event` wrapped in `Arc` so cloning per spawn is cheap
- Each listener gets its own `tokio::spawn` -- one slow/failing listener cannot block others
- Errors are logged and swallowed (fire-and-forget)
- Empty `listeners` vec means dispatch is a no-op (replaces the old `EventSource::Dev` skip logic)
- No serialization in the dispatcher -- events flow as typed data

### SQLite Listener

Moves from `data_access` to `event_processing/src/listeners/sqlite.rs`.

```rust
pub struct SqliteListener {
    conn: Arc<Mutex<Connection>>,
}

impl EventListener for SqliteListener {
    fn name(&self) -> &str { "sqlite" }

    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError> {
        let event_type = event.event_type().to_string();
        let payload = serde_json::to_string(event).unwrap(); // serialize at this boundary
        // INSERT INTO events (source, event_type, payload) VALUES (?, ?, ?)
    }
}
```

`query_events` remains as a concrete method on `SqliteListener` for the CLI `activity` tool. No trait needed for the read side.

### Telegram Listener

Lives in `event_processing/src/listeners/telegram.rs`.

```rust
pub struct TelegramListener {
    client: reqwest::Client,
    bot_token: String,
    chat_id: String,
}

impl EventListener for TelegramListener {
    fn name(&self) -> &str { "telegram" }

    async fn on_event(&self, source: &EventSource, event: &Event) -> Result<(), EventListenerError> {
        let message = self.format_message(source, event);
        // POST https://api.telegram.org/bot{token}/sendMessage
        // { "chat_id": self.chat_id, "text": message }
    }
}
```

`format_message` pattern-matches on the `Event` enum:
- `Event::Search(e)` -> show query, search type, result count, filters
- `Event::GetAbstract(e)` -> show which paper's abstract was accessed
- `Event::PaperOpen(e)` -> show paper_id and referrer
- Events the Telegram listener doesn't care about -> return `Ok(())` silently

### Configuration

New config section:

```toml
[event_processing.telegram]
bot_token = "123456:ABC-DEF..."
chat_id = "987654321"
```

Activity SQLite config moves from `[data_access.activity.sqlite]` to `[event_processing.activity.sqlite]`:

```toml
[event_processing.activity.sqlite]
filename = "data/activity.db"
```

### Wiring (configuration crate)

```rust
pub fn build_event_dispatcher(config: &AppConfig) -> Arc<EventDispatcher> {
    let mut dispatcher = EventDispatcher::new();

    if let Some(sqlite_cfg) = /* event_processing.activity.sqlite */ {
        info!("Registering SQLite event listener: {}", sqlite_cfg.filename);
        dispatcher.register(Arc::new(SqliteListener::new(sqlite_cfg)));
    }

    if let Some(telegram_cfg) = /* event_processing.telegram */ {
        info!("Registering Telegram event listener");
        dispatcher.register(Arc::new(TelegramListener::new(telegram_cfg)));
    }

    Arc::new(dispatcher)
}
```

### AppState Changes

Both `mcp` and `web` `AppState` structs replace:
```rust
// Before
pub activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,

// After
pub dispatcher: Arc<EventDispatcher>,
```

### Handler Signature Changes

Before:
```rust
pub async fn search(
    searcher: &Searcher,
    args: SearchArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> anyhow::Result<SearchResult> {
    // ...
    log_activity(activity_client, source, "search", serde_json::json!({...}));
}
```

After:
```rust
pub async fn search(
    searcher: &Searcher,
    args: SearchArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> anyhow::Result<SearchResult> {
    // ...
    dispatcher.dispatch(source, Event::Search(SearchEvent {
        query: args.query,
        search_type: search_type_str,
        result_count: search_result.chunks.len(),
        league_filter: args.league_filter,
        // ...
    }));
}
```

## Code Removals

- `data_access::activity` module entirely (trait, sqlite_client, config, mock)
- `api::activity` module (`log_activity` function, `EventSource` enum -- both move to `event_processing`)
- `ActivityConfig` from `data_access::config`
- `load_activity_client` from `configuration::helpers`
- `MockActivityClient` usage in api tests (replaced by a test `EventListener`)

## Dependency Graph Changes

Before:
```
api -> data_access (for ActivityClient trait)
configuration -> data_access (for ActivitySqliteClient)
```

After:
```
api -> event_processing (for EventDispatcher, EventSource, Event enum)
configuration -> event_processing (for building the dispatcher)
event_processing is self-contained (owns enum, dispatcher, all listeners)
data_access no longer has any activity-related code
```

## Testing

- **EventDispatcher**: unit test with a mock listener that records received events
- **SqliteListener**: unit test with `:memory:` SQLite (same pattern as today)
- **TelegramListener**: unit test that verifies message formatting per event variant (test `format_message` directly)
- **Handler integration**: test listener registered on a real dispatcher, verify correct event variant is dispatched
- **CLI activity tool**: still works via `SqliteListener::query_events` directly

## Migration

1. Create `event_processing` crate, add to workspace
2. Implement core types: `Event` enum, `EventSource`, `EventListener` trait, `EventDispatcher`
3. Implement `SqliteListener` (port from `data_access::activity`)
4. Implement `TelegramListener`
5. Add config types and `build_event_dispatcher` to `configuration`
6. Update `api` handlers to use dispatcher + `Event` enum
7. Update `mcp` and `web` `AppState` and wiring
8. Update CLI `activity` tool to use `SqliteListener` directly
9. Remove old `data_access::activity` module and `api::activity` module
10. Update `config.toml`, `config.docker.toml`, `CLAUDE.md`
