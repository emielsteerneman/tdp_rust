# Suggestions Endpoint Design

**Date**: 2026-03-22
**Status**: Approved

## Overview

Add a free-form suggestions endpoint accessible via both MCP and Web APIs. Users and LLMs can submit a text message as a suggestion. The suggestion is dispatched as an `Event` through the existing `EventDispatcher`, meaning it flows to all registered listeners (currently SQLite activity logging and Telegram notifications). No new storage, tables, traits, or config fields are introduced.

## Design

### Event Layer (`event_processing/src/lib.rs`)

New event struct and variant:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SuggestionEvent {
    pub message: String,
}

// Add to Event enum:
Event::Suggestion(SuggestionEvent)

// event_type() returns "suggestion"
```

### API Handler (`api/src/suggestion.rs`)

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestionArgs {
    #[schemars(description = "The suggestion or feedback message")]
    pub message: String,
}
```

- Trims whitespace, then validates message is non-empty
- Caps message at 2000 characters (Telegram Bot API limit is 4096; leave room for formatting)
- Dispatches `Event::Suggestion(SuggestionEvent { message })`
- Returns `String` acknowledgement on success, `ApiError` on validation failure

Register as `pub mod suggestion;` in `api/src/lib.rs`.

### MCP Tool (`mcp/src/server.rs`)

New `#[tool]` method `submit_suggestion`:
- Description explains that users/LLMs can leave free-form suggestions
- Calls `api::suggestion::submit_suggestion()` with `EventSource::Mcp`
- Returns success text via `CallToolResult::success`

### Web Route (`web/src/routes/suggestion.rs`)

- `POST /api/suggestion` accepting `Json<SuggestionArgs>`
- Calls `api::suggestion::submit_suggestion()` with `EventSource::Web`
- Returns JSON success response

Register in `web/src/routes/mod.rs`.

## What We're NOT Doing

- No new SQLite tables (existing activity listener logs it automatically)
- No new traits or client interfaces
- No new config fields
- No retrieval/listing endpoint
- No categories, tags, or contact info — just a message

## Files Changed

| File | Change |
|------|--------|
| `event_processing/src/lib.rs` | Add `SuggestionEvent` struct + `Event::Suggestion` variant + update existing tests |
| `api/src/suggestion.rs` | New file: handler + args |
| `api/src/lib.rs` | Add `pub mod suggestion;` |
| `mcp/src/server.rs` | Add `submit_suggestion` tool method |
| `web/src/routes/suggestion.rs` | New file: POST handler |
| `web/src/routes/mod.rs` | Register route |
| `frontend/src/lib/api.ts` | Add `submitSuggestion` function |
| `frontend/src/routes/suggestions/+page.svelte` | New file: suggestions form page |
| `frontend/src/lib/components/Navbar.svelte` | Add link to suggestions page |
| `CLAUDE.md` | Document the new endpoint |
