## Debugging Guidelines
When debugging issues, investigate the root cause before suggesting fixes. Don't suggest surface-level workarounds (e.g., switching package managers) without understanding why something is actually broken.

## Project Overview
RoboCup Team Description Paper (TDP) search and retrieval system. Indexes 2000+ annual technical
papers from RoboCup teams and exposes them via a hybrid semantic+keyword search engine through:
- MCP server (port 50001, open) and (port 50002, OAuth) for AI assistant integration
- REST API + SvelteKit web frontend (port 50000)
- CLI tools for corpus initialization and offline analysis

## Architecture
Cargo workspace organized in layers:
- `data_structures` — shared domain types, no I/O
- `data_access` — trait-defined storage abstractions (EmbedClient, VectorClient, MetadataClient)
- `data_processing` — chunking, IDF computation, hybrid search orchestration (`Searcher`)
- `event_processing` — typed `Event` enum, `EventDispatcher` (fan-out to listeners), `EventListener` trait, SQLite and Telegram listener implementations
- `configuration` — config loading (TOML + `TDP_*` env var overrides), client factories, `build_event_dispatcher`
- `api` — shared async handlers used by both `mcp` and `web`
- `mcp`, `web` — server crates (keep thin; business logic lives in `api`)
- `tools` — CLI binaries (initialize, create_idf, search_by_sentence, activity)
- `frontend/` — SvelteKit static site (built to `frontend/build/`)

Key architectural rule: both `mcp` and `web` call the same `api` handlers — don't duplicate logic.

## Key Conventions
- **TDP naming**: `{league}__{year}__{team}__{index}` (double underscore), e.g. `soccer_smallsize__2024__RoboTeam_Twente__0`
- **Dual name forms**: every `League` and `TeamName` has a machine name (`soccer_smallsize`) and pretty name (`Soccer SmallSize`). Both forms are accepted interchangeably by constructors/parsers.
- **Trait-based DI**: all external systems are behind async traits — switch implementations via config, not code changes.
- **`configuration::helpers`**: use these factory functions to instantiate clients; don't construct them directly in `main.rs`.
- **Fire-and-forget event dispatch**: `EventDispatcher::dispatch` spawns a task per listener; never blocks the caller. Handlers call `dispatcher.dispatch(source, Event::Variant(...))`.
- **EventListener implementations**: SQLite (stores events) and Telegram (sends notifications) are registered via config. Add new listeners by implementing `EventListener` and registering in `build_event_dispatcher`.

## Build & Run
```bash
# Backend
cargo build
cargo run -p mcp          # MCP servers: open on :50001, OAuth on :50002 (reads config.toml from cwd)
cargo run -p web          # Web server on :50000 (reads config.toml from cwd)
cargo run --bin initialize # Corpus ingestion pipeline
cargo test

# Frontend
cd frontend && npm install && npm run dev
npm run build             # -> frontend/build/

# Docker (full stack, requires qdrant.snapshot in repo root)
docker-compose up
```

## Local Setup Prerequisites
`config.toml` is gitignored — create it in the repo root before running anything. Minimum required fields:
```toml
[data_access]

[data_access.embed.openai]
model_name = "text-embedding-3-small"
api_key = "sk-..."

[data_access.vector.qdrant]
url = "http://localhost:6334"
embedding_size = 1536          # must match the embed model's output dimension

[data_access.metadata.sqlite]
filename = "data/metadata.db"

[data_processing]
tdps_markdown_root = "/path/to/tdps_markdown/"

[event_processing.activity.sqlite]
filename = "data/activity.db"

# Optional: Telegram notifications
# [event_processing.telegram]
# bot_token = "123456:ABC-DEF..."
# chat_id = "987654321"
```

Other prerequisites:
- **Qdrant** must be running at the configured URL before starting `mcp` or `web` (use `docker-compose up qdrant`)
- **TDP markdown files** must exist at `tdps_markdown_root` before running `initialize`
- **Static files**: the web server expects files at `./static/` (relative to cwd), but the frontend builds to `frontend/build/`. For local dev, symlink: `ln -s frontend/build static`
- **Embed model ↔ Qdrant size must match**: if you change the embed model, update `embedding_size` and re-run `initialize` to rebuild the Qdrant collection. Mismatches cause silent failures.

## Key Terms
- **lyti** — League Year Team Index. The canonical paper identifier used as a Qdrant payload field and in filters. Format: `soccer_smallsize__2024__RoboTeam_Twente__0`.
- **EventSource** — passed to all `api` handlers for event dispatch. Use `EventSource::Mcp` in the MCP server, `EventSource::Web` in the web server. If no listeners are registered, dispatch is a no-op.

## Adding a New Tool / Endpoint
Follow this pattern to keep both interfaces in sync:

1. **Add typed event** in `event_processing/src/lib.rs` — add a struct and `Event` enum variant.
2. **Add handler** in `api/src/<name>.rs` — takes typed args + clients + `&EventDispatcher` + `EventSource`, calls `dispatcher.dispatch(source, Event::Variant(...))`. Not all handlers need a client (e.g., `suggestion` only needs the dispatcher).
3. **MCP**: add a `#[tool(...)]` method in `mcp/src/server.rs` that calls the api handler with `&self.state.dispatcher, EventSource::Mcp`.
4. **Web**: add a route file `web/src/routes/<name>.rs` calling the api handler with `&state.dispatcher, EventSource::Web`, then register it in `web/src/routes/mod.rs`.

## Testing Approach
- Unit tests: in-file `#[cfg(test)]` modules throughout
- Mock-based tests: `mockall` with `#[automock]` on `MetadataClient`
- Integration tests: `testcontainers` spins up a real Qdrant Docker container (`qdrant/qdrant:v1.16`)
- Config tests: `tempfile` with temporary TOML files
- Integration tests require Docker to be available; they are NOT `#[ignore]`-gated
