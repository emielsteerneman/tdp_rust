## Why This Exists
RoboCup teams publish Technical Description Papers (TDPs) every year — 2000+ papers describing
their robot designs, strategies, and software. These papers are scattered, unsearchable, and
hard to cross-reference. This project makes them searchable through hybrid semantic+keyword
search, exposed via an MCP server for AI assistants and a web interface for humans.

## How It Works
Markdown TDPs are parsed, chunked, and embedded (dense + sparse vectors) by the `initialize`
pipeline, then stored in Qdrant. At query time, a `Searcher` generates embeddings for the query
and retrieves results via hybrid search. Both the MCP server and web server call shared `api`
handlers — they never duplicate business logic.

```
markdown files → data_processing (parse, chunk, embed) → data_access (store in Qdrant + SQLite)
                                                                ↓
                                    user query → api (shared handlers) → data_access (search Qdrant)
                                                   ↑                ↑
                                                  mcp              web → frontend (SvelteKit SPA)
```

## Crate Overview
- `data_structures` — Pure domain types (TDPName, League, Chunk, Filter). No I/O.
- `data_access` — Trait-defined storage abstractions and their implementations (Qdrant, SQLite, OpenAI).
- `data_processing` — Markdown parsing, text chunking, IDF computation, hybrid search orchestration.
- `event_processing` — Fire-and-forget event system: Event enum, EventDispatcher, SQLite and Telegram listeners.
- `configuration` — TOML config loading with `TDP_*` env overrides, factory functions for all clients.
- `api` — Shared async handlers used by both `mcp` and `web`. This is where business logic lives.
- `mcp` — MCP server (rmcp framework). Thin wrapper that calls `api` handlers. Dual ports: open (:50001) and OAuth (:50002).
- `web` — Axum HTTP server (:50000). Thin wrapper that calls `api` handlers. Serves the frontend SPA.
- `tools` — CLI binaries: `initialize`, `create_idf`, `search_by_sentence`, `activity`, `generate_team_code`, `set_team_metadata`.
- `frontend/` — SvelteKit static SPA. Talks to `web` via `/api/*` endpoints.
- `scripts/` — Qdrant maintenance shell scripts.
- `docs/` — Architecture diagrams and planning docs.

## Key Conventions
- **TDP naming**: `{league}__{year}__{team}__{index}` (double underscore), e.g. `soccer_smallsize__2024__RoboTeam_Twente__0`
- **Dual name forms**: every `League` and `TeamName` has a machine name (`soccer_smallsize`) and pretty name (`Soccer SmallSize`). Both forms are accepted interchangeably.
- **Trait-based DI**: all external systems are behind async traits — switch implementations via config, not code changes.
- **`configuration::helpers`**: use these factory functions to instantiate clients; don't construct them directly.
- **Shared handlers**: both `mcp` and `web` call `api` handlers — never duplicate logic between them.

## Build & Run
```bash
# Backend
cargo build
cargo run -p mcp          # MCP servers: open on :50001, OAuth on :50002
cargo run -p web          # Web server on :50000
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

# Optional: Team registry for team metadata (websites, repos, social links)
# [data_access.teams.sqlite]
# filename = "data/teams.db"
# master_password = "your-secret-here"  # only consumed on first DB init
```

Other prerequisites:
- **Qdrant** must be running at the configured URL before starting `mcp` or `web` (use `docker-compose up qdrant`)
- **TDP markdown files** must exist at `tdps_markdown_root` before running `initialize`
- **Static files**: the web server expects `./static/`; for local dev symlink: `ln -s frontend/build static`
- **Embed model ↔ Qdrant size must match**: if you change the embed model, update `embedding_size` and re-run `initialize`

## Key Terms
- **lyti** — League Year Team Index. Canonical paper identifier: `soccer_smallsize__2024__RoboTeam_Twente__0`.
- **EventSource** — passed to `api` handlers. Use `EventSource::Mcp` in MCP, `EventSource::Web` in web.

## Debugging Guidelines
When debugging issues, investigate the root cause before suggesting fixes. Don't suggest surface-level workarounds without understanding why something is actually broken.

## Testing Approach
- Unit tests: in-file `#[cfg(test)]` modules throughout
- Mock-based tests: `mockall` with `#[automock]` on `MetadataClient`
- Integration tests: `testcontainers` spins up a real Qdrant Docker container (`qdrant/qdrant:v1.16`)
- Config tests: `tempfile` with temporary TOML files
- Integration tests require Docker to be available; they are NOT `#[ignore]`-gated
