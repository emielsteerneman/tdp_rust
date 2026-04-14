## Purpose
MCP server using the `rmcp` framework. Thin wrapper — all logic lives in `api`.

## Structure
- `server.rs` — `AppServer` struct with `#[tool_router]` impl. Each `#[tool(...)]` method calls an `api` handler.
- `state.rs` — `AppState` holding Arc'd clients (metadata, searcher, dispatcher, registry, website_url).
- `oauth.rs` — PKCE OAuth flow with in-memory token store. Dynamic client registration, auto-approve.
- `main.rs` — dual server setup: open on :50001, OAuth-protected on :50002. Both share the same `AppServer`.

## Adding a New Tool
1. Add `#[tool(description = "...")]` method to `AppServer` impl in `server.rs`.
2. Method takes `Parameters<api::handler::ArgsType>`, calls `api::handler::handler_name()`.
3. Return `Ok(CallToolResult::success(vec![Content::text(response)]))` or `Err(McpError::internal_error(...))`.

## Tools (14)
`search`, `list_teams`, `list_leagues`, `list_years`, `list_papers`, `get_tdp_contents`,
`get_table_of_contents`, `get_section`, `get_abstract`, `get_references`, `get_paper_info`,
`get_team_info`, `get_league_info`, `submit_suggestion`.

## Notes
- `AppServer` must be `Clone` (rmcp creates one per session via `StreamableHttpService`).
- Expensive state is shared via Arc in `AppState`.
- Server instructions (for Claude clients) are defined at the bottom of `server.rs`.
