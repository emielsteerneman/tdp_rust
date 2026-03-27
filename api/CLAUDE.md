## Purpose
Shared async handlers called by both `mcp` and `web`. All business logic lives here.

## Handler Pattern
Every handler follows this signature:
```rust
pub async fn handler_name(
    client: Arc<dyn SomeClient>,
    args: HandlerArgs,            // derives Deserialize + JsonSchema
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<ReturnType, ApiError>
```
Handlers dispatch an event after processing, before returning.

## Error Types
- `ApiError::Argument(field, message)` — bad client input
- `ApiError::Forbidden(reason)` — auth failure
- `ApiError::Internal(message)` — server-side failure

## Shared Utilities
- `paper_filter.rs` — `PaperFilter` struct with `matches()` and `filter_papers()` for league/year/team filtering.
- `paper_navigation.rs` — `compute_section_range()` and `compute_breadcrumbs()` for TOC-based navigation.

## Adding a New Handler
1. Create `api/src/<name>.rs` with handler function + args struct.
2. Re-export from `lib.rs`.
3. Add event variant in `event_processing`.
4. Wire up in both `mcp/src/server.rs` and `web/src/routes/`.
