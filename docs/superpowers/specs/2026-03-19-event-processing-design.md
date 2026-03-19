# Event Processing System Design

## Problem

Activity logging is currently tightly coupled to a single SQLite backend via the `ActivityClient` trait. The user wants Telegram notifications when the API is used (to see what users and LLMs search for), and the system should support adding more listeners in the future without changing handler code.

## Goals

- Decouple event emission from event consumption
- Support multiple listeners (SQLite logging, Telegram notifications, future listeners)
- Type-safe event creation at the handler level
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
    lib.rs              -- Event, TypedEvent trait, EventListener trait, EventSource enum
    dispatcher.rs       -- EventDispatcher
    listeners/
      mod.rs
      sqlite.rs         -- SqliteListener (event storage + query)
      telegram.rs       -- TelegramListener
    events.rs           -- All 14 typed event structs
```

### Core Types

```rust
/// Source of the event (which server interface)
pub enum EventSource {
    Web,
    Mcp,
}

/// Untyped event as seen by listeners
pub struct Event {
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Implemented by typed event structs for compile-time safety
pub trait TypedEvent: Serialize + Send + 'static {
    fn event_type(&self) -> &'static str;
}

/// Implemented by all event consumers
#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_event(&self, event: &Event) -> Result<(), EventListenerError>;
    fn name(&self) -> &str;
}
```

### EventDispatcher

```rust
pub struct EventDispatcher {
    listeners: Vec<Arc<dyn EventListener>>,
}

impl EventDispatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, listener: Arc<dyn EventListener>);

    /// Serialize typed event to Value, then spawn a task per listener
    pub fn dispatch(&self, source: EventSource, event: impl TypedEvent) {
        let payload = serde_json::to_value(&event).unwrap();
        let event = Arc::new(Event {
            source: source.as_str().to_string(),
            event_type: event.event_type().to_string(),
            payload,
        });
        for listener in &self.listeners {
            let listener = listener.clone();
            let event = event.clone();
            tokio::spawn(async move {
                if let Err(e) = listener.on_event(&event).await {
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

### Typed Event Structs (14 total)

All live in `event_processing/src/events.rs`. Each implements `Serialize` + `TypedEvent`.

| Struct | event_type | Fields |
|--------|-----------|--------|
| `SearchEvent` | `"search"` | query, search_type, result_count, league_filter, year_filter, team_filter, content_type_filter |
| `ListLeaguesEvent` | `"list_leagues"` | result_count |
| `ListYearsEvent` | `"list_years"` | league, team, result_count |
| `ListTeamsEvent` | `"list_teams"` | hint, result_count |
| `ListPapersEvent` | `"list_papers"` | league, year, team, result_count |
| `GetAbstractEvent` | `"get_abstract"` | paper |
| `GetTableOfContentsEvent` | `"get_table_of_contents"` | paper |
| `GetSectionEvent` | `"get_section"` | paper, content_seq, include_children, items_returned |
| `GetParagraphEvent` | `"get_paragraph"` | paper, content_seq |
| `GetTableEvent` | `"get_table"` | paper, content_seq |
| `GetImageEvent` | `"get_image"` | paper, content_seq |
| `GetTdpContentsEvent` | `"get_tdp_contents"` | league, year, team |
| `HttpRequestEvent` | `"http_request"` | method, path, status, duration_ms |
| `PaperOpenEvent` | `"paper_open"` | paper_id, referrer |

### SQLite Listener

Moves from `data_access` to `event_processing/src/listeners/sqlite.rs`.

```rust
pub struct SqliteListener {
    conn: Arc<Mutex<Connection>>,
}

impl EventListener for SqliteListener {
    fn name(&self) -> &str { "sqlite" }

    async fn on_event(&self, event: &Event) -> Result<(), EventListenerError> {
        // INSERT INTO events (source, event_type, payload) VALUES (?, ?, ?)
        // payload serialized to string at this boundary
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

    async fn on_event(&self, event: &Event) -> Result<(), EventListenerError> {
        let message = self.format_message(event);
        // POST https://api.telegram.org/bot{token}/sendMessage
        // { "chat_id": self.chat_id, "text": message }
    }
}
```

`format_message` matches on `event.event_type`:
- Search events: show query, search type, result count, filters
- Content access events: show which paper/section was accessed
- List events: show what was listed and any filters
- Unknown event types: skip silently (return `Ok(())`)

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
    dispatcher.dispatch(source, SearchEvent {
        query: args.query,
        search_type: search_type_str,
        result_count: search_result.chunks.len(),
        league_filter: args.league_filter,
        // ...
    });
}
```

## Code Removals

- `data_access::activity` module entirely (trait, sqlite_client, config, mock)
- `api::activity` module (`log_activity` function, `EventSource` enum -- both move to `event_processing`)
- `ActivityConfig` from `data_access::config`
- `load_activity_client` from `configuration::helpers`
- `MockActivityClient` usage in api tests (replaced by a mock `EventDispatcher` or a test `EventListener`)

## Dependency Graph Changes

Before:
```
api -> data_access (for ActivityClient trait)
configuration -> data_access (for ActivitySqliteClient)
```

After:
```
api -> event_processing (for EventDispatcher, EventSource, typed events)
configuration -> event_processing (for building the dispatcher)
event_processing is self-contained (owns trait, dispatcher, all listeners)
data_access no longer has any activity-related code
```

## Testing

- **EventDispatcher**: unit test with a mock listener that records received events
- **SqliteListener**: unit test with `:memory:` SQLite (same pattern as today)
- **TelegramListener**: unit test that verifies HTTP request format (mock HTTP or just test `format_message`)
- **Typed events**: verify each implements `TypedEvent` and serializes correctly
- **Handler integration**: mock `EventListener` registered on a real dispatcher, verify correct typed event is dispatched
- **CLI activity tool**: still works via `SqliteListener::query_events` directly

## Migration

1. Create `event_processing` crate, add to workspace
2. Implement core types: `Event`, `TypedEvent`, `EventListener`, `EventSource`, `EventDispatcher`
3. Implement typed event structs
4. Implement `SqliteListener` (port from `data_access::activity`)
5. Implement `TelegramListener`
6. Add config types and `build_event_dispatcher` to `configuration`
7. Update `api` handlers to use dispatcher + typed events
8. Update `mcp` and `web` `AppState` and wiring
9. Update CLI `activity` tool to use `SqliteListener` directly
10. Remove old `data_access::activity` module and `api::activity` module
11. Update `config.toml`, `config.docker.toml`, `CLAUDE.md`
