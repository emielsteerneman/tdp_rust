## Debugging Guidelines
When debugging issues, investigate the root cause before suggesting fixes. Don't suggest surface-level workarounds (e.g., switching package managers) without understanding why something is actually broken.

## Project Overview
RoboCup Team Description Paper (TDP) search and retrieval system. Indexes 2000+ annual technical
papers from RoboCup teams and exposes them via a hybrid semantic+keyword search engine through:
- MCP server (port 8002) for AI assistant integration
- REST API + SvelteKit web frontend (port 8081)
- CLI tools for corpus initialization and offline analysis

## Architecture
Cargo workspace with 11 crates organized in layers:
- `data_structures` — shared domain types, no I/O
- `data_access` — trait-defined storage abstractions (EmbedClient, VectorClient, MetadataClient, ActivityClient)
- `data_processing` — chunking, IDF computation, hybrid search orchestration (`Searcher`)
- `configuration` — config loading (TOML + `TDP_*` env var overrides), client factories
- `api` — shared async handlers used by both `mcp` and `web`
- `mcp`, `web` — server crates (keep thin; business logic lives in `api`)
- `tools` — CLI binaries (initialize, create_idf, search_by_sentence, activity)
- `frontend/` — SvelteKit static site (built to `frontend/build/`)

Key architectural rule: both `mcp` and `web` call the same `api` handlers — don't duplicate logic.

## Key Conventions
- **TDP naming**: `{league}__{year}__{team}__{index}` (double underscore), e.g. `soccer_smallsize__2024__RoboTeam_Twente__0`
- **Dual name forms**: every `League` and `TeamName` has a machine name (`soccer_smallsize`) and pretty name (`Soccer SmallSize`). Qdrant payloads use pretty names; file keys use machine names.
- **Trait-based DI**: all external systems are behind async traits — switch implementations via config, not code changes.
- **`configuration::helpers`**: use these factory functions to instantiate clients; don't construct them directly in `main.rs`.
- **Fire-and-forget activity logging**: `api::activity::log_activity` always spawns a task; never blocks the caller.

## Build & Run
```bash
# Backend
cargo build
cargo run -p mcp          # MCP server (reads config.toml from cwd)
cargo run -p web          # Web server (reads config.toml from cwd)
cargo run --bin initialize # Corpus ingestion pipeline
cargo test

# Frontend
cd frontend && npm install && npm run dev
npm run build             # -> frontend/build/

# Docker (full stack, requires qdrant.snapshot in repo root)
docker-compose up
```

## Testing Approach
- Unit tests: in-file `#[cfg(test)]` modules throughout
- Mock-based tests: `mockall` with `#[automock]` on `MetadataClient` and `ActivityClient`
- Integration tests: `testcontainers` spins up a real Qdrant Docker container (`qdrant/qdrant:v1.16`)
- Config tests: `tempfile` with temporary TOML files
- Integration tests require Docker to be available; they are NOT `#[ignore]`-gated
