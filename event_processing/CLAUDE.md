## Purpose
Fire-and-forget event system for tracking usage across MCP and web interfaces.

## Architecture
- `Event` enum — 20 variants covering all user actions (Search, GetAbstract, GetReferences, PaperOpen, Suggestion, etc.).
- `EventSource` — Web or Mcp. Passed through `api` handlers to identify the caller.
- `EventDispatcher` — holds Vec of listeners. `dispatch()` spawns a tokio task per listener; never blocks the caller.
- `EventListener` trait — `on_event(&self, source, event)`. Failures are logged, not propagated.

## Listeners
- `SqliteListener` — stores all events as JSON payloads in SQLite. Supports querying with filters (source, type, since, limit).
- `TelegramListener` — sends formatted messages for "interesting" events. Skips noisy ones (list operations, HTTP requests).

## Adding a New Event
1. Add a struct and `Event` enum variant in `lib.rs`.
2. Update `event_type()` match arm.
3. If Telegram-relevant, add formatting in `TelegramListener::format_message()`.
