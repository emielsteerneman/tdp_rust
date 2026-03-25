# Team Registry: Design Spec

## Problem

TDPs describe algorithms and approaches, but often lack enough detail to implement them. Many teams publish their source code on GitHub/GitLab, but there's no connection between the TDP search system and those codebases. A user reading about TIGERs Mannheim's bang-bang trajectory planner has no way to find the actual implementation at `github.com/TIGERs-Mannheim/Sumatra`.

## Goal

Allow teams to associate metadata (websites, code repositories, social links) with their team profile, so that TDP search users (both humans and LLMs via MCP) can discover a team's source code and other resources alongside their papers.

## Non-Goals

- Indexing or vectorizing code from repositories (future work)
- Automated discovery of GitHub repos from TDP content
- Frontend team profile pages (follow-up work; MCP + API first)

## Approach

Extend `data_access` with a new `teams` sub-module following the existing trait-based DI pattern. Team metadata is stored in a separate SQLite database (`data/teams.db`), keeping user-writable data isolated from the read-only paper corpus.

## Data Model

### SQLite Schema (`data/teams.db`)

```sql
CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Stores:
--   "salt"        -> random hex string, generated on first init
--   "master_hash" -> HMAC-SHA256(salt, master_password)

CREATE TABLE team_metadata (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    team_name  TEXT NOT NULL,   -- machine name (e.g. "TIGERs_Mannheim")
    key        TEXT NOT NULL,   -- e.g. "website", "github", "gitlab", "twitter"
    value      TEXT NOT NULL,   -- the URL or value
    updated_at TEXT NOT NULL    -- ISO 8601 timestamp
);

CREATE INDEX idx_team_metadata_team ON team_metadata(team_name);
```

Key properties:
- `team_name` uses machine name format (underscores) for consistency with the rest of the system
- No unique constraint on `(team_name, key)` — a team can have multiple values for the same key (e.g. multiple GitHub repos). API responses return an array of entries, not a map, to preserve duplicates.
- The `config` table stores the HMAC salt and master password hash. The DB is fully self-contained and portable.

### Domain Types

```rust
pub struct TeamMetadataEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,  // ISO 8601
}
```

The `id` column is internal to SQLite and not exposed.

### Error Types

```rust
pub enum TeamRegistryError {
    Internal(String),
    NotFound(String),
}
```

Auth and validation failures are handled at the API handler level, not in the storage trait. The trait returns `NotFound` if the team has no entries, `Internal` for DB errors.

### Authentication

- **Team code**: `HMAC-SHA256(salt, team_machine_name)`, hex-encoded, truncated to first 16 characters. Deterministic — can be regenerated from the DB at any time.
- **Master password**: Provided via config on first init. Stored as `HMAC-SHA256(salt, master_password)` in the `config` table. The plaintext is never stored.
- **Master password init behavior**: The `master_password` config field is only consumed when the DB has no `master_hash` row yet (first init). On subsequent startups, the field is ignored. To rotate the master password, delete the `master_hash` row from the `config` table and restart with the new password. This is an admin-only operation.
- **Verification**: When a code is submitted, the system checks:
  1. Does it match the truncated `HMAC-SHA256(salt, team_name)` for the target team? (team access)
  2. Does `HMAC-SHA256(salt, submitted_code)` match the stored `master_hash`? (admin access — the submitted code is the raw master password)
  3. Either match permits the write.
- **Timing safety**: Both comparisons must use constant-time equality (`subtle::ConstantTimeEq` or equivalent) to prevent timing side-channel attacks.

### Input Validation

- Maximum 50 entries per team
- Maximum key length: 64 characters, alphanumeric + underscores only
- Maximum value length: 2048 characters
- No URL format validation — values are free-form strings (teams may want to store non-URL values)
- Validation is enforced in `update_team_info` API handler, not in the storage layer

## Module Structure

### data_access/src/teams/

```
data_access/src/teams/
├── mod.rs            -- TeamRegistryClient trait + error types + TeamMetadataEntry
└── sqlite_client.rs  -- SQLite implementation
```

**Trait definition** (follows existing `<'a>` lifetime convention from `MetadataClient`):

```rust
pub trait TeamRegistryClient: Send + Sync {
    fn get_team_metadata<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<Vec<TeamMetadataEntry>, TeamRegistryError>> + Send + 'a>>;

    fn set_team_metadata<'a>(&'a self, team_name: &'a str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), TeamRegistryError>> + Send + 'a>>;

    fn verify_code<'a>(&'a self, team_name: &'a str, code: &'a str)
        -> Pin<Box<dyn Future<Output = Result<bool, TeamRegistryError>> + Send + 'a>>;

    fn generate_team_code<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<String, TeamRegistryError>> + Send + 'a>>;
}
```

`set_team_metadata` does a full replace (delete existing + insert new) within a single SQLite transaction. If any insert fails, the transaction rolls back and the team's existing data is preserved. The web form sends all current entries at once.

### Configuration

New **optional** section in `config.toml`:

```toml
[data_access.teams.sqlite]
filename = "data/teams.db"
master_password = "your-secret-here"  # only consumed on first init
```

If the `[data_access.teams.sqlite]` section is absent, the team registry feature is disabled. The `get_team_info` MCP tool and web routes are not registered, and the servers start without this capability. This follows the same pattern as the optional `[event_processing.telegram]` section.

A new factory function `build_team_registry_client()` in `configuration/src/helpers.rs` returns `Option<Arc<dyn TeamRegistryClient>>`.

### State Integration

Both `mcp/src/state.rs` and `web/src/state.rs` gain an `Option<Arc<dyn TeamRegistryClient>>` field. If `None`, the MCP tool is not registered and the web routes return 404.

## API Handlers

### `api/src/get_team_info.rs` (read, public)

- Input: team name (pretty or machine format)
- Output: `Vec<TeamMetadataEntry>` — array of `{ key, value, updated_at }` objects
- Returns empty array if team has no entries (not an error)
- No auth required
- Dispatches `GetTeamInfo` event

### `api/src/update_team_info.rs` (write, code-protected)

- Input: team name, code, list of `(key, value)` entries
- Validates input (entry count, key/value length, key format)
- Verifies code via `TeamRegistryClient::verify_code`
- On success: calls `set_team_metadata`, dispatches `UpdateTeamInfo` event
- On auth failure: returns error (HTTP 403 Forbidden)
- On validation failure: returns error (HTTP 400 Bad Request)

## MCP Integration

One new read-only tool (only registered when team registry is configured):

```
get_team_info(team: String) -> team metadata
```

Description: "Get a team's website, code repositories, and other metadata. Use after finding relevant TDPs to discover the team's source code and online presence. A team may have multiple entries for the same key (e.g. multiple GitHub repos)."

No MCP tool for updates — teams use the web page. LLMs should not modify team data.

## Web Routes

Only registered when team registry is configured:

- `GET /api/team-registry/:name` — returns team metadata as JSON array of `{ key, value, updated_at }` objects. HTTP 200 always (empty array if no entries).
- `POST /api/team-registry/:name` — accepts `{ code: "...", entries: [{ key: "...", value: "..." }, ...] }`. Returns HTTP 200 on success, 403 on auth failure, 400 on validation failure.
- Static page at `/teams/edit` — simple form: paste team name + code, edit key-value pairs, submit

The `/api/team-registry/` prefix avoids collision with the existing `/api/teams` route used by `list_teams`.

## Events

```rust
pub struct GetTeamInfoEvent {
    pub team: String,
}

pub struct UpdateTeamInfoEvent {
    pub team: String,
    pub entries: Vec<(String, String)>,
}
```

Added as `Event::GetTeamInfo` and `Event::UpdateTeamInfo` variants.

## CLI Tooling

New binary in `tools/`: `generate_team_code`

```
cargo run --bin generate_team_code -- --team "TIGERs Mannheim"
# outputs: Code for TIGERs_Mannheim: a3f8c1d2e9b04f17
```

Takes a team name (pretty or machine), loads `config.toml` from the current working directory (same convention as other CLI tools in `tools/`), calls `build_team_registry_client()` to get the client, then calls `TeamRegistryClient::generate_team_code` and prints the result. No HMAC logic is duplicated outside the client.

The `teams.db` is auto-initialized (tables created, salt generated) on first access by the SQLite client — no separate init step needed.

## Implementation Notes

- Work on a separate feature branch
- Follow the existing pattern: add tool/endpoint via `api` handler, wire into both `mcp` and `web`
- Frontend edit page is a simple form — no complex UI needed
- MCP tool is the highest-value deliverable; prioritize it
