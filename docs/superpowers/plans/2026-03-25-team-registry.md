# Team Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow teams to associate metadata (websites, code repos, social links) with their profile, discoverable via MCP and web API.

**Architecture:** New `data_access/src/teams/` module with its own SQLite DB, trait-based DI, wired into `api`, `mcp`, `web`, and a CLI tool. Optional feature — disabled if config section is absent.

**Tech Stack:** Rust, SQLite (rusqlite), HMAC-SHA256 (sha2 + hmac crates), subtle (constant-time comparison), axum, rmcp, SvelteKit

**Spec:** `docs/superpowers/specs/2026-03-25-team-registry-design.md`

**Branch:** Create feature branch `feature/team-registry` before starting.

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `data_access/src/teams/mod.rs` | `TeamRegistryClient` trait, `TeamRegistryError`, `TeamMetadataEntry` |
| Create | `data_access/src/teams/sqlite_client.rs` | SQLite implementation with HMAC auth |
| Modify | `data_access/src/lib.rs` | Add `pub mod teams;` |
| Modify | `data_access/Cargo.toml` | Add `hmac`, `sha2`, `subtle`, `hex`, `rand` deps |
| Modify | `data_access/src/config.rs` | Add `TeamsConfig` with optional sqlite field |
| Modify | `configuration/src/helpers.rs` | Add `build_team_registry_client()` |
| Modify | `configuration/src/appconfig.rs` | Add `teams` field to `DataAccessConfig` |
| Create | `api/src/get_team_info.rs` | Public read handler |
| Create | `api/src/update_team_info.rs` | Code-protected write handler |
| Modify | `api/src/lib.rs` | Add `pub mod get_team_info; pub mod update_team_info;` |
| Modify | `event_processing/src/lib.rs` | Add `GetTeamInfoEvent`, `UpdateTeamInfoEvent`, enum variants |
| Modify | `mcp/src/state.rs` | Add `Option<Arc<dyn TeamRegistryClient>>` |
| Modify | `mcp/src/server.rs` | Add `get_team_info` tool |
| Modify | `mcp/src/main.rs` | Call `build_team_registry_client()`, pass to state |
| Modify | `web/src/state.rs` | Add `Option<Arc<dyn TeamRegistryClient>>` |
| Create | `web/src/routes/team_registry.rs` | GET + POST handlers |
| Modify | `web/src/routes/mod.rs` | Register team registry routes |
| Modify | `web/src/main.rs` | Call `build_team_registry_client()`, pass to state |
| Create | `tools/src/bin/generate_team_code.rs` | CLI to generate team codes |
| Create | `frontend/src/routes/teams/edit/+page.svelte` | Team metadata edit form |
| Modify | `frontend/src/lib/api.ts` | Add `getTeamInfo()` and `updateTeamInfo()` |

---

### Task 1: Create feature branch

- [ ] **Step 1: Create and switch to feature branch**

```bash
git checkout -b feature/team-registry
```

- [ ] **Step 2: Verify branch**

Run: `git branch --show-current`
Expected: `feature/team-registry`

---

### Task 2: TeamRegistryClient trait and types

**Files:**
- Create: `data_access/src/teams/mod.rs`
- Modify: `data_access/src/lib.rs`
- Modify: `data_access/Cargo.toml`

- [ ] **Step 1: Add dependencies to data_access/Cargo.toml**

Add these lines to `[dependencies]`:

```toml
hmac = "0.12"
sha2 = "0.10"
subtle = "2.6"
hex = "0.4"
rand = "0.9"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Create data_access/src/teams/mod.rs with trait, error, and entry types**

```rust
mod sqlite_client;
pub use sqlite_client::{TeamsSqliteClient, TeamsSqliteConfig};

use std::future::Future;
use std::pin::Pin;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum TeamRegistryError {
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetadataEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

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

- [ ] **Step 3: Add `pub mod teams;` to data_access/src/lib.rs**

Add `pub mod teams;` alongside the existing module declarations.

- [ ] **Step 4: Create empty data_access/src/teams/sqlite_client.rs stub**

Create a minimal stub so the crate compiles:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TeamsSqliteConfig {
    pub filename: String,
    pub master_password: Option<String>,
}

pub struct TeamsSqliteClient;
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p data_access`
Expected: compiles with warnings about unused imports/types

- [ ] **Step 6: Commit**

```bash
git add data_access/
git commit -m "feat: add TeamRegistryClient trait and types"
```

---

### Task 3: SQLite client implementation

**Files:**
- Modify: `data_access/src/teams/sqlite_client.rs`

- [ ] **Step 1: Write tests for the SQLite client**

Add at the bottom of `sqlite_client.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::teams::TeamRegistryClient;

    fn test_client() -> TeamsSqliteClient {
        TeamsSqliteClient::new(TeamsSqliteConfig {
            filename: ":memory:".to_string(),
            master_password: Some("test-master-pw".to_string()),
        })
    }

    #[tokio::test]
    async fn test_get_empty_team() {
        let client = test_client();
        let entries = client.get_team_metadata("NonExistent_Team").await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_set_and_get_metadata() {
        let client = test_client();
        let entries = vec![
            ("website".to_string(), "https://example.com".to_string()),
            ("github".to_string(), "https://github.com/example".to_string()),
        ];
        client.set_team_metadata("Test_Team", entries).await.unwrap();

        let result = client.get_team_metadata("Test_Team").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "website");
        assert_eq!(result[0].value, "https://example.com");
        assert_eq!(result[1].key, "github");
        assert_eq!(result[1].value, "https://github.com/example");
    }

    #[tokio::test]
    async fn test_set_replaces_existing() {
        let client = test_client();
        client.set_team_metadata("Test_Team", vec![
            ("website".to_string(), "https://old.com".to_string()),
        ]).await.unwrap();

        client.set_team_metadata("Test_Team", vec![
            ("website".to_string(), "https://new.com".to_string()),
            ("github".to_string(), "https://github.com/new".to_string()),
        ]).await.unwrap();

        let result = client.get_team_metadata("Test_Team").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].value, "https://new.com");
    }

    #[tokio::test]
    async fn test_set_empty_clears_entries() {
        let client = test_client();
        client.set_team_metadata("Test_Team", vec![
            ("website".to_string(), "https://example.com".to_string()),
        ]).await.unwrap();

        client.set_team_metadata("Test_Team", vec![]).await.unwrap();

        let result = client.get_team_metadata("Test_Team").await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_values_same_key() {
        let client = test_client();
        client.set_team_metadata("Test_Team", vec![
            ("github".to_string(), "https://github.com/repo1".to_string()),
            ("github".to_string(), "https://github.com/repo2".to_string()),
        ]).await.unwrap();

        let result = client.get_team_metadata("Test_Team").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "github");
        assert_eq!(result[1].key, "github");
    }

    #[tokio::test]
    async fn test_teams_are_isolated() {
        let client = test_client();
        client.set_team_metadata("Team_A", vec![
            ("website".to_string(), "https://a.com".to_string()),
        ]).await.unwrap();
        client.set_team_metadata("Team_B", vec![
            ("website".to_string(), "https://b.com".to_string()),
        ]).await.unwrap();

        let a = client.get_team_metadata("Team_A").await.unwrap();
        let b = client.get_team_metadata("Team_B").await.unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(a[0].value, "https://a.com");
        assert_eq!(b[0].value, "https://b.com");
    }

    #[tokio::test]
    async fn test_verify_team_code() {
        let client = test_client();
        let code = client.generate_team_code("Test_Team").await.unwrap();

        assert!(client.verify_code("Test_Team", &code).await.unwrap());
        assert!(!client.verify_code("Test_Team", "wrong_code").await.unwrap());
        assert!(!client.verify_code("Other_Team", &code).await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_master_password() {
        let client = test_client();

        // Master password works for any team
        assert!(client.verify_code("Any_Team", "test-master-pw").await.unwrap());
        assert!(client.verify_code("Other_Team", "test-master-pw").await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_wrong_master_password() {
        let client = test_client();
        // Only the exact master password works, not arbitrary strings
        assert!(!client.verify_code("Test_Team", "wrong-master-pw").await.unwrap());
    }

    #[tokio::test]
    async fn test_generate_team_code_deterministic() {
        let client = test_client();
        let code1 = client.generate_team_code("Test_Team").await.unwrap();
        let code2 = client.generate_team_code("Test_Team").await.unwrap();
        assert_eq!(code1, code2);
        assert_eq!(code1.len(), 16); // truncated to 16 hex chars
    }

    #[tokio::test]
    async fn test_different_teams_different_codes() {
        let client = test_client();
        let code_a = client.generate_team_code("Team_A").await.unwrap();
        let code_b = client.generate_team_code("Team_B").await.unwrap();
        assert_ne!(code_a, code_b);
    }

    #[tokio::test]
    async fn test_salt_persists_across_instances() {
        // Use a temp file instead of :memory: to test persistence
        let dir = std::env::temp_dir().join(format!("teams_test_{}", std::process::id()));
        let db_path = dir.join("teams.db");
        std::fs::create_dir_all(&dir).unwrap();

        let config = TeamsSqliteConfig {
            filename: db_path.to_str().unwrap().to_string(),
            master_password: Some("test-pw".to_string()),
        };

        let code1 = {
            let client = TeamsSqliteClient::new(config.clone());
            client.generate_team_code("Test_Team").await.unwrap()
        };

        let code2 = {
            let client = TeamsSqliteClient::new(TeamsSqliteConfig {
                master_password: None, // not first init, should be ignored
                ..config
            });
            client.generate_team_code("Test_Team").await.unwrap()
        };

        assert_eq!(code1, code2);

        // Cleanup
        std::fs::remove_dir_all(&dir).ok();
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p data_access teams`
Expected: compilation errors (TeamsSqliteClient doesn't implement the trait yet)

- [ ] **Step 3: Implement the SQLite client**

Replace the stub in `data_access/src/teams/sqlite_client.rs` with the full implementation:

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use rand::Rng;
use rusqlite::{Connection, params};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::teams::{TeamMetadataEntry, TeamRegistryClient, TeamRegistryError};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize, Clone)]
pub struct TeamsSqliteConfig {
    pub filename: String,
    pub master_password: Option<String>,
}

pub struct TeamsSqliteClient {
    conn: Arc<Mutex<Connection>>,
    salt: String,
}

impl TeamsSqliteClient {
    pub fn new(config: TeamsSqliteConfig) -> Self {
        let conn = Connection::open(&config.filename)
            .expect("Failed to open teams SQLite database");

        conn.query_row("PRAGMA journal_mode=WAL;", [], |_| Ok(()))
            .expect("Failed to set WAL mode");

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )", [],
        ).expect("Failed to create config table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS team_metadata (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                team_name  TEXT NOT NULL,
                key        TEXT NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )", [],
        ).expect("Failed to create team_metadata table");

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_team_metadata_team ON team_metadata(team_name)",
            [],
        ).expect("Failed to create index");

        // Initialize or load salt
        let salt: String = conn.query_row(
            "SELECT value FROM config WHERE key = 'salt'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| {
            let new_salt = hex::encode(rand::random::<[u8; 32]>());
            conn.execute(
                "INSERT INTO config (key, value) VALUES ('salt', ?1)",
                params![new_salt],
            ).expect("Failed to store salt");
            new_salt
        });

        // Initialize master hash if master_password provided and no hash stored yet
        if let Some(ref master_pw) = config.master_password {
            let has_master: bool = conn.query_row(
                "SELECT COUNT(*) FROM config WHERE key = 'master_hash'",
                [],
                |row| row.get::<_, i64>(0),
            ).map(|c| c > 0).unwrap_or(false);

            if !has_master {
                let master_hash = Self::compute_hmac(&salt, master_pw);
                conn.execute(
                    "INSERT INTO config (key, value) VALUES ('master_hash', ?1)",
                    params![master_hash],
                ).expect("Failed to store master hash");
            }
        }

        Self {
            conn: Arc::new(Mutex::new(conn)),
            salt,
        }
    }

    fn compute_hmac(salt: &str, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn compute_team_code(salt: &str, team_name: &str) -> String {
        let full_hmac = Self::compute_hmac(salt, team_name);
        full_hmac[..16].to_string()
    }

    fn constant_time_eq(a: &str, b: &str) -> bool {
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }
}

impl TeamRegistryClient for TeamsSqliteClient {
    fn get_team_metadata<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<Vec<TeamMetadataEntry>, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let conn = self.conn.lock()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let mut stmt = conn.prepare(
                "SELECT key, value, updated_at FROM team_metadata WHERE team_name = ?1 ORDER BY id"
            ).map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let entries = stmt.query_map(params![team_name], |row| {
                Ok(TeamMetadataEntry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(|e| TeamRegistryError::Internal(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            Ok(entries)
        })
    }

    fn set_team_metadata<'a>(&'a self, team_name: &'a str, entries: Vec<(String, String)>)
        -> Pin<Box<dyn Future<Output = Result<(), TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let conn = self.conn.lock()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let tx = conn.unchecked_transaction()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            tx.execute("DELETE FROM team_metadata WHERE team_name = ?1", params![team_name])
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let now = chrono::Utc::now().to_rfc3339();
            for (key, value) in &entries {
                tx.execute(
                    "INSERT INTO team_metadata (team_name, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                    params![team_name, key, value, now],
                ).map_err(|e| TeamRegistryError::Internal(e.to_string()))?;
            }

            tx.commit()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            Ok(())
        })
    }

    fn verify_code<'a>(&'a self, team_name: &'a str, code: &'a str)
        -> Pin<Box<dyn Future<Output = Result<bool, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            // Check team code
            let expected_team_code = Self::compute_team_code(&self.salt, team_name);
            if Self::constant_time_eq(code, &expected_team_code) {
                return Ok(true);
            }

            // Check master password
            let conn = self.conn.lock()
                .map_err(|e| TeamRegistryError::Internal(e.to_string()))?;

            let master_hash: Option<String> = conn.query_row(
                "SELECT value FROM config WHERE key = 'master_hash'",
                [],
                |row| row.get(0),
            ).ok();

            if let Some(stored_hash) = master_hash {
                let submitted_hash = Self::compute_hmac(&self.salt, code);
                if Self::constant_time_eq(&submitted_hash, &stored_hash) {
                    return Ok(true);
                }
            }

            Ok(false)
        })
    }

    fn generate_team_code<'a>(&'a self, team_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<String, TeamRegistryError>> + Send + 'a>>
    {
        Box::pin(async move {
            Ok(Self::compute_team_code(&self.salt, team_name))
        })
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p data_access teams`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add data_access/
git commit -m "feat: implement TeamsSqliteClient with HMAC auth"
```

---

### Task 4: Configuration wiring

**Files:**
- Modify: `data_access/src/config.rs`
- Modify: `configuration/src/appconfig.rs`
- Modify: `configuration/src/helpers.rs`

- [ ] **Step 1: Add TeamsConfig and update DataAccessConfig in data_access/src/config.rs**

Add the import at the top:

```rust
use crate::teams::TeamsSqliteConfig;
```

Add the `TeamsConfig` struct after `MetadataConfig`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct TeamsConfig {
    pub sqlite: Option<TeamsSqliteConfig>,
}
```

Add the `teams` field to `DataAccessConfig`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct DataAccessConfig {
    pub embed: EmbedConfig,
    pub vector: VectorConfig,
    pub metadata: MetadataConfig,
    pub teams: Option<TeamsConfig>,
}
```

- [ ] **Step 2: Add build_team_registry_client to configuration/src/helpers.rs**

Add import and function:

```rust
use data_access::teams::{TeamRegistryClient, TeamsSqliteClient};
```

```rust
pub fn build_team_registry_client(config: &AppConfig) -> Option<Arc<dyn TeamRegistryClient + Send + Sync>> {
    let teams_config = config.data_access.teams.as_ref()?;
    let sqlite_cfg = teams_config.sqlite.as_ref()?;

    info!("Using SQLite Teams Registry with file: {}", sqlite_cfg.filename);
    Some(Arc::new(TeamsSqliteClient::new(sqlite_cfg.clone())))
}
```

- [ ] **Step 4: Verify existing config tests still pass**

Run: `cargo test -p configuration`
Expected: all tests pass (the new `teams` field is `Option`, so existing TOML without it deserializes fine)

- [ ] **Step 5: Commit**

```bash
git add data_access/src/config.rs configuration/
git commit -m "feat: add team registry configuration wiring"
```

---

### Task 5: Event types

**Files:**
- Modify: `event_processing/src/lib.rs`

- [ ] **Step 1: Add event structs**

Add after `SuggestionEvent`:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct GetTeamInfoEvent {
    pub team: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTeamInfoEvent {
    pub team: String,
    pub entries: Vec<(String, String)>,
}
```

- [ ] **Step 2: Add enum variants**

Add to the `Event` enum:

```rust
GetTeamInfo(GetTeamInfoEvent),
UpdateTeamInfo(UpdateTeamInfoEvent),
```

- [ ] **Step 3: Add event_type match arms**

Add to `Event::event_type()`:

```rust
Event::GetTeamInfo(_) => "get_team_info",
Event::UpdateTeamInfo(_) => "update_team_info",
```

- [ ] **Step 4: Add test cases to test_event_type_strings**

Add to the test vector:

```rust
(Event::GetTeamInfo(GetTeamInfoEvent { team: "t".into() }), "get_team_info"),
(Event::UpdateTeamInfo(UpdateTeamInfoEvent { team: "t".into(), entries: vec![] }), "update_team_info"),
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p event_processing`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add event_processing/
git commit -m "feat: add GetTeamInfo and UpdateTeamInfo events"
```

---

### Task 6: API handlers

**Files:**
- Create: `api/src/get_team_info.rs`
- Create: `api/src/update_team_info.rs`
- Modify: `api/src/lib.rs`

- [ ] **Step 1: Add Forbidden variant to api/src/error.rs**

Add a new variant to the `ApiError` enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Argument error: {0} : {1}")]
    Argument(String, String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
```

- [ ] **Step 2: Create api/src/get_team_info.rs**

```rust
use std::sync::Arc;

use data_access::teams::{TeamRegistryClient, TeamMetadataEntry};
use data_structures::file::TeamName;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetTeamInfoEvent};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTeamInfoArgs {
    #[schemars(description = "Team name (e.g. 'TIGERs Mannheim' or 'TIGERs_Mannheim')")]
    pub team: String,
}

pub async fn get_team_info(
    team_registry: Arc<dyn TeamRegistryClient + Send + Sync>,
    args: GetTeamInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> anyhow::Result<Vec<TeamMetadataEntry>> {
    let team_name = TeamName::new(args.team.trim());

    let entries = team_registry
        .get_team_metadata(&team_name.name)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    dispatcher.dispatch(
        source,
        Event::GetTeamInfo(GetTeamInfoEvent {
            team: team_name.name.clone(),
        }),
    );

    Ok(entries)
}
```

- [ ] **Step 2: Create api/src/update_team_info.rs**

```rust
use std::sync::Arc;

use data_access::teams::TeamRegistryClient;
use data_structures::file::TeamName;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, UpdateTeamInfoEvent};
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct UpdateTeamInfoArgs {
    pub code: String,
    pub entries: Vec<UpdateEntry>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntry {
    pub key: String,
    pub value: String,
}

pub async fn update_team_info(
    team_registry: Arc<dyn TeamRegistryClient + Send + Sync>,
    team: &str,
    args: UpdateTeamInfoArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<String, ApiError> {
    let team_name = TeamName::new(team.trim());

    // Validate entries
    if args.entries.len() > 50 {
        return Err(ApiError::Argument(
            "entries".to_string(),
            "Maximum 50 entries per team".to_string(),
        ));
    }

    for entry in &args.entries {
        if entry.key.len() > 64 {
            return Err(ApiError::Argument(
                "key".to_string(),
                format!("Key '{}' exceeds 64 character limit", entry.key),
            ));
        }
        if !entry.key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ApiError::Argument(
                "key".to_string(),
                format!("Key '{}' contains invalid characters (alphanumeric and underscores only)", entry.key),
            ));
        }
        if entry.value.len() > 2048 {
            return Err(ApiError::Argument(
                "value".to_string(),
                format!("Value for key '{}' exceeds 2048 character limit", entry.key),
            ));
        }
    }

    // Verify auth
    let authorized = team_registry
        .verify_code(&team_name.name, &args.code)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !authorized {
        return Err(ApiError::Forbidden("Invalid team code".to_string()));
    }

    let entries_tuples: Vec<(String, String)> = args.entries
        .into_iter()
        .map(|e| (e.key, e.value))
        .collect();

    let entries_for_event = entries_tuples.clone();

    team_registry
        .set_team_metadata(&team_name.name, entries_tuples)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::UpdateTeamInfo(UpdateTeamInfoEvent {
            team: team_name.name.clone(),
            entries: entries_for_event,
        }),
    );

    Ok("Team metadata updated successfully".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_processing::dispatcher::EventDispatcher;

    fn mock_args(code: &str, entries: Vec<(&str, &str)>) -> UpdateTeamInfoArgs {
        UpdateTeamInfoArgs {
            code: code.to_string(),
            entries: entries.into_iter().map(|(k, v)| UpdateEntry {
                key: k.to_string(),
                value: v.to_string(),
            }).collect(),
        }
    }

    #[tokio::test]
    async fn test_validation_too_many_entries() {
        let entries: Vec<(&str, &str)> = (0..51).map(|_| ("key", "value")).collect();
        let args = mock_args("code", entries);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(matches!(result, Err(ApiError::Argument(ref f, _)) if f == "entries"));
    }

    #[tokio::test]
    async fn test_validation_key_too_long() {
        let long_key = "a".repeat(65);
        let args = mock_args("code", vec![(&long_key, "value")]);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(matches!(result, Err(ApiError::Argument(ref f, _)) if f == "key"));
    }

    #[tokio::test]
    async fn test_validation_key_invalid_chars() {
        let args = mock_args("code", vec![("invalid-key", "value")]);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(matches!(result, Err(ApiError::Argument(ref f, _)) if f == "key"));
    }

    #[tokio::test]
    async fn test_validation_value_too_long() {
        let long_value = "a".repeat(2049);
        let args = mock_args("code", vec![("key", &long_value)]);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(matches!(result, Err(ApiError::Argument(ref f, _)) if f == "value"));
    }

    #[tokio::test]
    async fn test_auth_failure() {
        let args = mock_args("wrong_code", vec![("website", "https://example.com")]);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(matches!(result, Err(ApiError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_successful_update_with_master_password() {
        let args = mock_args("master-pw", vec![("website", "https://example.com")]);
        let registry = Arc::new(data_access::teams::TeamsSqliteClient::new(
            data_access::teams::TeamsSqliteConfig {
                filename: ":memory:".to_string(),
                master_password: Some("master-pw".to_string()),
            },
        ));

        let result = update_team_info(
            registry, "team", args, &EventDispatcher::new(), EventSource::Web,
        ).await;

        assert!(result.is_ok());
    }
}
```

- [ ] **Step 3: Add modules to api/src/lib.rs**

Add:
```rust
pub mod get_team_info;
pub mod update_team_info;
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p api`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add api/src/
git commit -m "feat: add get_team_info and update_team_info API handlers"
```

---

### Task 7: MCP integration

**Files:**
- Modify: `mcp/src/state.rs`
- Modify: `mcp/src/server.rs`
- Modify: `mcp/src/main.rs`

- [ ] **Step 1: Add team registry to MCP state**

In `mcp/src/state.rs`, add:

```rust
use data_access::teams::TeamRegistryClient;
```

Add field to `AppState`:
```rust
pub team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
```

Update `AppState::new` to accept and store the new field.

- [ ] **Step 2: Add get_team_info tool to MCP server**

In `mcp/src/server.rs`, add the tool method inside the `#[tool_router] impl AppServer` block:

```rust
#[tool(
    description = "Get a team's website, code repositories, and other metadata. Use after finding relevant TDPs to discover the team's source code and online presence. A team may have multiple entries for the same key (e.g. multiple GitHub repos)."
)]
pub async fn get_team_info(
    &self,
    Parameters(args): Parameters<api::get_team_info::GetTeamInfoArgs>,
) -> Result<CallToolResult, McpError> {
    let registry = self.state.team_registry.as_ref()
        .ok_or_else(|| McpError::internal_error("Team registry not configured".to_string(), None))?;

    match api::get_team_info::get_team_info(
        registry.clone(),
        args,
        &self.state.dispatcher,
        event_processing::EventSource::Mcp,
    ).await {
        Ok(entries) => match serde_json::to_string_pretty(&entries) {
            Ok(response) => Ok(CallToolResult::success(vec![Content::text(response)])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        },
        Err(e) => Err(McpError::internal_error(e.to_string(), None)),
    }
}
```

Add the `get_team_info` import at the top of the file with the other api imports.

- [ ] **Step 3: Wire team registry in MCP main.rs**

In `mcp/src/main.rs`, after the `dispatcher` line, add:

```rust
let team_registry = configuration::helpers::build_team_registry_client(&config);
```

Update the `AppState::new` call to pass `team_registry`.

- [ ] **Step 4: Verify it compiles**

Run: `cargo check -p mcp`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add mcp/
git commit -m "feat: add get_team_info MCP tool"
```

---

### Task 8: Web routes

**Files:**
- Modify: `web/src/state.rs`
- Create: `web/src/routes/team_registry.rs`
- Modify: `web/src/routes/mod.rs`
- Modify: `web/src/main.rs`
- Modify: `web/src/error.rs`

- [ ] **Step 1: Add team registry to web state**

In `web/src/state.rs`, add:

```rust
use data_access::teams::TeamRegistryClient;
```

Add field:
```rust
pub team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
```

Update `AppState::new` to accept and store the new field.

- [ ] **Step 2: Add forbidden helper to web/src/error.rs**

Add method to `ApiError` impl:

```rust
pub fn forbidden(message: impl Into<String>) -> Self {
    Self::new(StatusCode::FORBIDDEN, message)
}
```

- [ ] **Step 3: Create web/src/routes/team_registry.rs**

```rust
use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;
use data_access::teams::TeamMetadataEntry;

pub async fn get_team_info_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<Vec<TeamMetadataEntry>>>, ApiError> {
    let registry = state.team_registry.as_ref()
        .ok_or_else(|| ApiError::not_found("Team registry not configured"))?;

    let args = api::get_team_info::GetTeamInfoArgs { team: name };

    let entries = api::get_team_info::get_team_info(
        registry.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(ApiResponse::new(entries)))
}

pub async fn update_team_info_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(args): Json<api::update_team_info::UpdateTeamInfoArgs>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let registry = state.team_registry.as_ref()
        .ok_or_else(|| ApiError::not_found("Team registry not configured"))?;

    let result = api::update_team_info::update_team_info(
        registry.clone(),
        &name,
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| match e {
        api::error::ApiError::Forbidden(_) => ApiError::forbidden(e.to_string()),
        api::error::ApiError::Argument(_, _) => ApiError::bad_request(e.to_string()),
        api::error::ApiError::Internal(_) => ApiError::internal_server_error(e.to_string()),
    })?;

    Ok(Json(ApiResponse::new(result)))
}
```

- [ ] **Step 4: Register routes in web/src/routes/mod.rs**

Add `mod team_registry;` to module declarations.

Add routes inside `create_router` in the `api_routes` builder:

```rust
.route("/api/team-registry/{name}", get(team_registry::get_team_info_handler)
    .post(team_registry::update_team_info_handler))
```

- [ ] **Step 5: Wire team registry in web/src/main.rs**

After the `dispatcher` line, add:

```rust
let team_registry = configuration::helpers::build_team_registry_client(&config);
```

Update the `AppState::new` call to pass `team_registry`.

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p web`
Expected: compiles

- [ ] **Step 7: Commit**

```bash
git add web/
git commit -m "feat: add team registry web routes"
```

---

### Task 9: CLI tool

**Files:**
- Create: `tools/src/bin/generate_team_code.rs`

- [ ] **Step 1: Create the CLI binary**

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let team_name = if let Some(pos) = args.iter().position(|a| a == "--team") {
        args.get(pos + 1)
            .ok_or_else(|| anyhow::anyhow!("--team requires a value"))?
            .clone()
    } else {
        anyhow::bail!("Usage: generate_team_code --team \"Team Name\"");
    };

    let config_path = "config.toml";
    let config = configuration::AppConfig::load_from_file(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config from {}: {}", config_path, e))?;

    let registry = configuration::helpers::build_team_registry_client(&config)
        .ok_or_else(|| anyhow::anyhow!(
            "Team registry not configured. Add [data_access.teams.sqlite] to config.toml"
        ))?;

    let team = data_structures::file::TeamName::new(&team_name);
    let code = registry.generate_team_code(&team.name).await?;

    println!("Code for {}: {}", team.name, code);

    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --bin generate_team_code`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add tools/
git commit -m "feat: add generate_team_code CLI tool"
```

---

### Task 10: Frontend edit page

**Files:**
- Modify: `frontend/src/lib/api.ts`
- Create: `frontend/src/routes/teams/edit/+page.svelte`

- [ ] **Step 1: Add API functions to frontend/src/lib/api.ts**

Add types and functions:

```typescript
export interface TeamMetadataEntry {
	key: string;
	value: string;
	updated_at: string;
}

export async function getTeamInfo(name: string, fetchFn?: FetchFn): Promise<TeamMetadataEntry[]> {
	return fetchApi<TeamMetadataEntry[]>(`/team-registry/${encodeURIComponent(name)}`, fetchFn);
}

export async function updateTeamInfo(
	name: string,
	code: string,
	entries: { key: string; value: string }[],
	fetchFn?: FetchFn
): Promise<string> {
	return fetchApi<string>(`/team-registry/${encodeURIComponent(name)}`, fetchFn, {
		method: 'POST',
		body: JSON.stringify({ code, entries })
	});
}
```

- [ ] **Step 2: Create frontend/src/routes/teams/edit/+page.svelte**

Create the load-then-edit page. This is a simple form that:
1. Has team name + code inputs and a "Load" button
2. Fetches existing entries on load
3. Displays entries as editable key-value rows with add/remove buttons
4. Submits the full set on save

```svelte
<script lang="ts">
	import { getTeamInfo, updateTeamInfo, type TeamMetadataEntry } from '$lib/api';

	let teamName = '';
	let code = '';
	let entries: { key: string; value: string }[] = [];
	let loaded = false;
	let loading = false;
	let saving = false;
	let message = '';
	let error = '';

	async function load() {
		if (!teamName.trim()) return;
		loading = true;
		error = '';
		message = '';
		try {
			const result = await getTeamInfo(teamName);
			entries = result.map((e) => ({ key: e.key, value: e.value }));
			loaded = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		} finally {
			loading = false;
		}
	}

	function addEntry() {
		entries = [...entries, { key: '', value: '' }];
	}

	function removeEntry(index: number) {
		entries = entries.filter((_, i) => i !== index);
	}

	async function save() {
		if (!code.trim()) {
			error = 'Please enter your team code';
			return;
		}
		saving = true;
		error = '';
		message = '';
		try {
			const result = await updateTeamInfo(teamName, code, entries);
			message = result;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Edit Team Info</title>
</svelte:head>

<div class="container">
	<h1>Edit Team Info</h1>

	<div class="load-section">
		<label>
			Team Name
			<input type="text" bind:value={teamName} placeholder="e.g. TIGERs Mannheim" />
		</label>
		<label>
			Team Code
			<input type="password" bind:value={code} placeholder="Your team code" />
		</label>
		<button onclick={load} disabled={loading || !teamName.trim()}>
			{loading ? 'Loading...' : 'Load'}
		</button>
	</div>

	{#if loaded}
		<div class="entries-section">
			<h2>Metadata</h2>
			{#each entries as entry, i}
				<div class="entry-row">
					<input type="text" bind:value={entry.key} placeholder="key (e.g. github)" />
					<input type="text" bind:value={entry.value} placeholder="value (e.g. https://github.com/...)" />
					<button class="remove" onclick={() => removeEntry(i)}>Remove</button>
				</div>
			{/each}
			<button onclick={addEntry}>+ Add Entry</button>

			<div class="save-section">
				<button onclick={save} disabled={saving}>
					{saving ? 'Saving...' : 'Save Changes'}
				</button>
			</div>
		</div>
	{/if}

	{#if message}
		<div class="message success">{message}</div>
	{/if}
	{#if error}
		<div class="message error">{error}</div>
	{/if}
</div>

<style>
	.container {
		max-width: 700px;
		margin: 2rem auto;
		padding: 0 1rem;
	}
	.load-section, .entries-section, .save-section {
		margin: 1.5rem 0;
	}
	label {
		display: block;
		margin-bottom: 0.5rem;
	}
	input[type='text'], input[type='password'] {
		width: 100%;
		padding: 0.5rem;
		margin-top: 0.25rem;
		box-sizing: border-box;
	}
	.entry-row {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		align-items: center;
	}
	.entry-row input:first-child {
		flex: 1;
	}
	.entry-row input:nth-child(2) {
		flex: 3;
	}
	button {
		padding: 0.5rem 1rem;
		cursor: pointer;
		margin-top: 0.5rem;
	}
	.remove {
		background: #c33;
		color: white;
		border: none;
		padding: 0.4rem 0.8rem;
	}
	.message {
		padding: 0.75rem;
		margin-top: 1rem;
		border-radius: 4px;
	}
	.success {
		background: #d4edda;
		color: #155724;
	}
	.error {
		background: #f8d7da;
		color: #721c24;
	}
</style>
```

- [ ] **Step 3: Verify frontend builds**

Run: `cd frontend && npm run build`
Expected: builds successfully

- [ ] **Step 4: Commit**

```bash
git add frontend/
git commit -m "feat: add team metadata edit page and API functions"
```

---

### Task 11: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Add team registry documentation**

Add to the "Local Setup Prerequisites" section in `config.toml` example:

```toml
# Optional: Team registry for team metadata (websites, repos, social links)
# [data_access.teams.sqlite]
# filename = "data/teams.db"
# master_password = "your-secret-here"  # only consumed on first DB init
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add team registry config to CLAUDE.md"
```

---

### Task 12: Full integration test

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 2: Run frontend build**

Run: `cd frontend && npm run build`
Expected: builds successfully

- [ ] **Step 3: Fix any issues and commit**

If there are failures, fix them and commit the fixes.
