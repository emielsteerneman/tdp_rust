## Purpose
Shared async handlers called by both `mcp` and `web`. All business logic lives here.

## Handler Pattern
Most handlers follow this signature:
```rust
pub async fn handler_name(
    metadata_client: Arc<dyn MetadataClient>,
    args: HandlerArgs,            // derives Deserialize + JsonSchema
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<ReturnType, ApiError>
```
Exceptions:
- `search()` takes `&Searcher` (not Arc'd trait) and returns `anyhow::Result<SearchResult>`.
- `get_team_info()` / `update_team_info()` / `get_league_info()` take `Arc<dyn RegistryClient>`.
- `submit_suggestion()` has no client parameter (only args, dispatcher, source).

Handlers dispatch an event after processing, before returning. `search_type` defaults to Hybrid when omitted.

## Error Types
- `ApiError::Argument(field, message)` — bad client input
- `ApiError::Forbidden(reason)` — auth failure
- `ApiError::Internal(message)` — server-side failure

## Handler Modules
`get_abstract`, `get_image`, `get_league_info`, `get_paper_info`, `get_paragraph`,
`get_references`, `get_section`, `get_table`, `get_table_of_contents`, `get_tdp_contents`,
`get_team_info`, `list_leagues`, `list_papers`, `list_teams`, `list_years`, `search`,
`suggestion`, `update_team_info`.

## Shared Utilities
- `paper_filter.rs` — `PaperFilter` struct with `matches()` and `filter_papers()` for league/year/team filtering.
- `paper_navigation.rs` — `compute_section_range()` and `compute_breadcrumbs()` for TOC-based section navigation with `BreadcrumbEntry` results.

## Adding a New Handler
1. Create `api/src/<name>.rs` with handler function + args struct.
2. Re-export from `lib.rs`.
3. Add event variant in `event_processing`.
4. Wire up in both `mcp/src/server.rs` and `web/src/routes/`.
