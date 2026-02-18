# TDP Search

Search engine for RoboCup Team Description Papers (TDPs). Hybrid dense + sparse search over 2000+ papers across all RoboCup leagues.

Live at [tdpsearch.com](https://tdpsearch.com).

## Architecture

Rust workspace with the following crates:

| Crate | Type | Description |
|---|---|---|
| `web` | Binary | Axum HTTP API server |
| `mcp` | Binary | MCP (Model Context Protocol) server for LLM integration |
| `frontend` | SvelteKit | Web UI with Tailwind CSS |
| `api` | Library | Shared business logic (search, list, filter) |
| `data_access` | Library | Trait-based clients: Qdrant, SQLite, OpenAI, FastEmbed |
| `data_processing` | Library | Chunking, embedding, IDF, search orchestration |
| `data_structures` | Library | Shared types (TDP, League, Chunk, Filter) |
| `configuration` | Library | Config loading and client initialization |
| `tools` | Binaries | CLI tools for initialization, search, and analytics |
| `chat` | Library | (experimental) Chat/conversation support |

## Getting Started

### Prerequisites

- Rust (edition 2024)
- Docker (for Qdrant)
- Node.js 22+ (for frontend)

### Setup

1. Start Qdrant vector database:
   ```
   make qdrant-restart
   ```

2. Configure `config.toml` with your embedding provider and paths.

3. Initialize the database (parse TDPs, compute embeddings, build IDF):
   ```
   make init
   ```

4. Start the web server and frontend:
   ```
   make web   # API server on :8081
   make ui    # SvelteKit dev server on :8003
   ```

### Docker

```
make docker       # build and start all services
make docker-logs  # follow logs
make docker-down  # stop
```

#### Setup

Before building Docker images, create your configuration files:

```bash
# Create Docker config from example
cp config.docker.toml.example config.docker.toml
# Edit config.docker.toml and add your OpenAI API key (or leave empty and use env vars)

# Create .env file for runtime overrides
cp .env.example .env
# Edit .env and configure your API keys
```

#### Configuration

Docker images bake in `config.docker.toml` as the default config. Settings can be overridden at runtime via environment variables using the `TDP_` prefix and `__` (double underscore) as a separator for nested keys.

For example, to set the OpenAI API key:
```
TDP_DATA_ACCESS__EMBED__OPENAI__API_KEY=sk-proj-...
```

There are two ways to pass environment variables to the containers:

1. **`env_file` directive** in `docker-compose.yml` — add `env_file: .env` to a service to inject all variables from `.env` directly into the container.

2. **`environment` block with interpolation** — reference host/`.env` variables using `${VAR}` syntax in docker-compose.yml. Note: Docker Compose auto-loads `.env` only for `${...}` interpolation within the compose file itself, it does **not** automatically pass `.env` variables into containers.

To get started, copy the example env file and fill in your values:
```
cp .env.example .env
```

#### Startup order

The `mcp` and `web` services use `depends_on` with a health check on Qdrant's `/healthz` endpoint. They will not start until Qdrant is ready to accept connections. If you need to rebuild after changing `config.docker.toml`, run `docker compose up --build` since the config is copied into the image at build time.

## CLI Tools

All CLI tools live in the `tools` crate and are run via `cargo run -p tools --bin <name>`.

### initialize

Parses TDP JSON files, computes embeddings, builds IDF, and upserts everything into Qdrant + SQLite.

```
make init
# or: cargo run --release -p tools --bin initialize
```

### create_idf

Recomputes the IDF (inverse document frequency) map from existing chunks without re-embedding.

```
cargo run -p tools --bin create_idf
```

### repl

Interactive search REPL for testing queries from the terminal.

```
cargo run -p tools --bin repl
```

### search_by_sentence

Runs a predefined set of sentence-level searches for benchmarking/testing.

```
cargo run -p tools --bin search_by_sentence
```

### activity

Query the activity log database for usage reports and scraper detection.

```
cargo run -p tools --bin activity -- summary              # event counts by type/source, top queries
cargo run -p tools --bin activity -- summary --since 2025-06-01
cargo run -p tools --bin activity -- recent               # last 20 events
cargo run -p tools --bin activity -- recent --limit 50
cargo run -p tools --bin activity -- agents               # user-agent and IP breakdown
cargo run -p tools --bin activity -- agents --since 2025-06-01
```

Or via Make:
```
make activity ARGS="summary"
make activity ARGS="agents --since 2025-06-01"
```

## Makefile Targets

**Services:**

| Target | Description |
|---|---|
| `make web` | Start the Axum API server on :8081 |
| `make mcp` | Start the MCP server |
| `make ui` | Start the SvelteKit dev server on :8003 |

**Tools:**

| Target | Description |
|---|---|
| `make init` | Initialize database (parse, embed, index) |
| `make create-idf` | Recompute IDF map |
| `make repl` | Interactive search REPL |
| `make search-by-sentence` | Run sentence-level search benchmarks |
| `make activity ARGS="..."` | Run the activity analytics CLI |

**Infrastructure:**

| Target | Description |
|---|---|
| `make qdrant-restart` | Restart Qdrant Docker container |
| `make docker` | Build and start all services via Docker Compose |
| `make docker-logs` | Follow Docker Compose logs |
| `make docker-down` | Stop Docker Compose |
| `make clean` | Restart Qdrant and delete SQLite databases |
| `make leagues` | Quick API test: list all leagues |

## MCP Server

The MCP server exposes TDP search functionality to LLMs. Available tools:

- `search` - Semantic search across all TDPs
- `get_tdp_contents` - Retrieve full markdown of a specific paper
- `list_papers` - List papers with optional league/year/team filters
- `list_teams` - List team names with optional hint filter
- `list_leagues` - List all RoboCup leagues
- `list_years` - List years with optional league/year/team filters

```
cargo run -p mcp
```

## Activity Logging

All interactions (searches, paper opens, list operations) are logged to `activity.db` from both Web and MCP sources. HTTP requests from the web server also capture IP and user-agent for scraper detection.

Configure in `config.toml`:
```toml
[data_access.activity.sqlite]
filename = "activity.db"
```
