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
- No unique constraint on `(team_name, key)` — a team can have multiple values for the same key (e.g. multiple GitHub repos)
- The `config` table stores the HMAC salt and master password hash. The DB is fully self-contained and portable.

### Authentication

- **Team code**: `HMAC-SHA256(salt, team_machine_name)`, hex-encoded, truncated to first 16 characters. Deterministic — can be regenerated from the DB at any time.
- **Master password**: Provided via config on first init. Stored as `HMAC-SHA256(salt, master_password)` in the `config` table. The plaintext is never stored.
- **Verification**: When a code is submitted, the system checks:
  1. Does it match `HMAC-SHA256(salt, team_name)` for the target team? (team access)
  2. Does `HMAC-SHA256(salt, submitted_code)` match the stored `master_hash`? (admin access)
  3. Either match permits the write.

## Module Structure

### data_access/src/teams/

```
data_access/src/teams/
├── mod.rs            -- TeamRegistryClient trait + error types
└── sqlite_client.rs  -- SQLite implementation
```

**Trait definition:**

```rust
pub trait TeamRegistryClient: Send + Sync {
    // Read: all key-value pairs for a team
    fn get_team_metadata(&self, team_name: &str)
        -> Pin<Box<dyn Future<Output = Result<Vec<TeamMetadataEntry>, TeamRegistryError>> + Send + '_>>;

    // Write: replace all entries for a team (delete + insert)
    fn set_team_metadata(&self, team_name: &str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), TeamRegistryError>> + Send + '_>>;

    // Auth: verify a team code or master password
    fn verify_code(&self, team_name: &str, code: &str)
        -> Pin<Box<dyn Future<Output = Result<bool, TeamRegistryError>> + Send + '_>>;

    // Admin: generate the code for a team
    fn generate_team_code(&self, team_name: &str)
        -> Pin<Box<dyn Future<Output = Result<String, TeamRegistryError>> + Send + '_>>;
}
```

`set_team_metadata` does a full replace (delete existing + insert new). The web form sends all current entries at once. Simpler than individual add/remove operations.

### Configuration

New section in `config.toml`:

```toml
[data_access.teams.sqlite]
filename = "data/teams.db"
master_password = "your-secret-here"  # read on first init, stored as HMAC
```

A new factory function `build_team_registry_client()` in `configuration/src/helpers.rs` following the existing pattern.

## API Handlers

### `api/src/get_team_info.rs` (read, public)

- Input: team name (pretty or machine format)
- Output: list of `(key, value)` entries
- No auth required
- Dispatches `GetTeamInfo` event

### `api/src/update_team_info.rs` (write, code-protected)

- Input: team name, code, list of `(key, value)` entries
- Verifies code via `TeamRegistryClient::verify_code`
- On success: calls `set_team_metadata`, dispatches `UpdateTeamInfo` event
- On failure: returns auth error

## MCP Integration

One new read-only tool:

```
get_team_info(team: String) -> team metadata
```

Description: "Get a team's website, code repositories, and other metadata. Use after finding relevant TDPs to discover the team's source code and online presence."

No MCP tool for updates — teams use the web page. LLMs should not modify team data.

## Web Routes

- `GET /api/teams/:name` — returns team metadata as JSON
- `POST /api/teams/:name` — accepts `{ code: "...", entries: [{ key: "...", value: "..." }, ...] }`, returns success or auth error
- Static page at `/teams/edit` — simple form: paste team name + code, edit key-value pairs, submit

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

Takes a team name (pretty or machine), loads the DB, outputs the HMAC code. This is how codes are generated to hand out to teams.

The `teams.db` is auto-initialized (tables created, salt generated) on first access by the SQLite client — no separate init step needed.

## Implementation Notes

- Work on a separate feature branch
- Follow the existing pattern: add tool/endpoint via `api` handler, wire into both `mcp` and `web`
- Frontend edit page is a simple form — no complex UI needed
- MCP tool is the highest-value deliverable; prioritize it
