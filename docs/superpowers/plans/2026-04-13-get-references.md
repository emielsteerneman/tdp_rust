# Get References Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `get_references` API endpoint that returns the bibliography of a TDP, backed by a new SQLite `reference` table.

**Architecture:** Fix the markdown parser to handle numbered (`N. `) and bracket (`[N]`) reference formats. Store parsed references in a new `reference` table during ingestion. Expose via a new `load_references` trait method, API handler, MCP tool, and web route — all following the existing `get_abstract` pattern.

**Tech Stack:** Rust, SQLite (rusqlite), rmcp, Axum, mockall

**Spec:** `docs/superpowers/specs/2026-04-13-get-references-design.md`

---

### Task 1: Fix markdown parser to handle numbered references

**Files:**
- Modify: `data_processing/src/markdown_parser.rs:389-397` (Section::References match arm)
- Modify: `data_processing/src/markdown_parser.rs:554-577` (existing test)

- [ ] **Step 1: Update the existing test to use numbered format**

In `data_processing/src/markdown_parser.rs`, find the test `test_parse_paragraphs` (around line 533). Change the references section from `* [1] Some reference` to numbered format, and add a second reference:

```rust
// In the test markdown string, replace:
//   # references
//   * [1] Some reference
// with:
# references
1. Some reference
2. Another reference
```

And update the assertions at the end of the test:

```rust
assert_eq!(tdp.references.len(), 2);
assert_eq!(tdp.references[0], "Some reference");
assert_eq!(tdp.references[1], "Another reference");
```

- [ ] **Step 2: Add a test for bracket format**

Add a new test in the same `#[cfg(test)]` module:

```rust
#[test]
fn test_parse_references_bracket_format() {
    let md = "\
# title
Test
# references
[1] First reference
[2] Second reference with URL https://example.com
";
    let tdp = parse_markdown(md, make_name());

    assert_eq!(tdp.references.len(), 2);
    assert_eq!(tdp.references[0], "First reference");
    assert_eq!(tdp.references[1], "Second reference with URL https://example.com");
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p data_processing -- test_parse_paragraphs test_parse_references_bracket_format`

Expected: Both tests FAIL — the parser only handles `* ` prefix.

- [ ] **Step 4: Fix the parser's References section handler**

In `data_processing/src/markdown_parser.rs`, replace the `Section::References` match arm (around line 389-397):

```rust
Section::References => {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        continue;
    }
    // Match "N. text" format (e.g. "1. Author, Title...")
    if let Some(after_num) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()).and_then(|rest| {
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        rest.strip_prefix(". ")
    }) {
        let after_num = after_num.trim();
        if !after_num.is_empty() {
            references.push(after_num.to_string());
        }
    }
    // Match "[N] text" format (e.g. "[1] Author, Title...")
    else if trimmed.starts_with('[') {
        if let Some(bracket_end) = trimmed.find(']') {
            let inside = &trimmed[1..bracket_end];
            if inside.chars().all(|c| c.is_ascii_digit()) {
                let after = trimmed[bracket_end + 1..].trim();
                if !after.is_empty() {
                    references.push(after.to_string());
                }
            }
        }
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p data_processing -- test_parse_paragraphs test_parse_references_bracket_format`

Expected: Both PASS.

- [ ] **Step 6: Commit**

```bash
git add data_processing/src/markdown_parser.rs
git commit -m "fix: parse numbered and bracket reference formats in markdown parser

Previously only matched '* ' prefix (1 paper). Now handles 'N. ' (1875 papers)
and '[N]' (191 papers) formats."
```

---

### Task 2: Add `reference` table to SQLite and store references during ingestion

**Files:**
- Modify: `data_access/src/metadata/sqlite_client.rs:55-104` (schema creation in `new()`)
- Modify: `data_access/src/metadata/sqlite_client.rs:382-462` (`store_paper`)
- Modify: `data_access/src/metadata/sqlite_client.rs:900-1019` (test)

- [ ] **Step 1: Add the `reference` table to schema creation**

In `data_access/src/metadata/sqlite_client.rs`, after the `toc_entry` table creation (around line 97), add:

```rust
conn.execute(
    "CREATE TABLE IF NOT EXISTS reference (
        paper_lyt TEXT NOT NULL,
        seq INTEGER NOT NULL,
        text TEXT NOT NULL,
        FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt),
        UNIQUE(paper_lyt, seq)
    )",
    [],
)
.expect("Failed to create table reference");
```

- [ ] **Step 2: Add DELETE FROM reference to the upsert cleanup in `store_paper`**

In the `store_paper` method (around line 406-411), add the delete after the existing deletes:

```rust
// Existing deletes:
tx.execute("DELETE FROM toc_entry WHERE paper_lyt = ?1", params![paper_lyt])
    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
tx.execute("DELETE FROM author WHERE paper_lyt = ?1", params![paper_lyt])
    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
// Add this new delete:
tx.execute("DELETE FROM reference WHERE paper_lyt = ?1", params![paper_lyt])
    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
tx.execute("DELETE FROM paper WHERE paper_lyt = ?1", params![paper_lyt])
    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
```

- [ ] **Step 3: Insert references in `store_paper`**

After the `toc_stmt` block (after `drop(toc_stmt);`, around line 451), add:

```rust
// Insert references
let mut ref_stmt = tx
    .prepare("INSERT INTO reference (paper_lyt, seq, text) VALUES (?1, ?2, ?3)")
    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

for (i, ref_text) in tdp.references.iter().enumerate() {
    ref_stmt
        .execute(params![paper_lyt, i as u32, ref_text])
        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
}
drop(ref_stmt);
```

- [ ] **Step 4: Update test data and add `load_references` test assertions**

In the `test_store_and_load_paper` test, update the existing references field in the test `MarkdownTDP` (around line 956) to reflect what the fixed parser would produce (prefix stripped):

```rust
references: vec!["Some Reference".to_string()],
```

Then after the abstract test (around line 1008), add:

```rust
// Test load_references
let refs = client
    .load_references(paper_lyt.clone())
    .await
    .expect("Failed to load references");
assert_eq!(refs.len(), 1);
assert_eq!(refs[0], "Some Reference");

// Test load_references for nonexistent paper returns empty vec
let no_refs = client
    .load_references("nonexistent__paper".to_string())
    .await
    .expect("Should return empty vec, not error");
assert!(no_refs.is_empty());
```

- [ ] **Step 5: Run the test to verify it fails (load_references doesn't exist yet)**

Run: `cargo test -p data_access -- test_store_and_load_paper`

Expected: FAIL — compile error, `load_references` method doesn't exist.

- [ ] **Step 6: Add `load_references` to the `MetadataClient` trait**

In `data_access/src/metadata/mod.rs`, add after the `load_paper_info` method (around line 87):

```rust
fn load_references<'a>(
    &'a self,
    paper_lyt: String,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>, MetadataClientError>> + Send + 'a>>;
```

- [ ] **Step 7: Implement `load_references` in `SqliteClient`**

In `data_access/src/metadata/sqlite_client.rs`, add a new method implementation (after the `load_paper_info` method):

```rust
fn load_references<'a>(
    &'a self,
    paper_lyt: String,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>, MetadataClientError>> + Send + 'a>> {
    let conn = self.conn.clone();

    Box::pin(async move {
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT text FROM reference WHERE paper_lyt = ?1 ORDER BY seq")
                .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

            let refs: Vec<String> = stmt
                .query_map(params![paper_lyt], |row| row.get(0))
                .map_err(|e| MetadataClientError::Internal(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

            Ok(refs)
        })
        .await
        .map_err(|e| MetadataClientError::Internal(e.to_string()))?
    })
}
```

- [ ] **Step 8: Run the test to verify it passes**

Run: `cargo test -p data_access -- test_store_and_load_paper`

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add data_access/src/metadata/mod.rs data_access/src/metadata/sqlite_client.rs
git commit -m "feat: store and load paper references in SQLite

Add reference table, write references during store_paper, add
load_references method to MetadataClient trait."
```

---

### Task 3: Add `GetReferences` event variant

**Files:**
- Modify: `event_processing/src/lib.rs`
- Modify: `event_processing/src/listeners/telegram.rs`

- [ ] **Step 1: Add the event struct and enum variant**

In `event_processing/src/lib.rs`, add the struct after `GetLeagueInfoEvent` (around line 150):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct GetReferencesEvent {
    pub paper: String,
}
```

Add the variant to the `Event` enum (after `UpdateTeamInfo`, around line 184):

```rust
GetReferences(GetReferencesEvent),
```

Add the match arm to `event_type()` (after `UpdateTeamInfo`, around line 209):

```rust
Event::GetReferences(_) => "get_references",
```

- [ ] **Step 2: Add to test vectors**

In `test_event_type_strings` (around line 280), add:

```rust
(Event::GetReferences(GetReferencesEvent { paper: "p".into() }), "get_references"),
```

In `test_event_serialization_all_variants` (around line 322), add:

```rust
Event::GetReferences(GetReferencesEvent { paper: "test_paper".into() }),
```

- [ ] **Step 3: Add Telegram formatting**

In `event_processing/src/listeners/telegram.rs`, add a match arm in `format_message` after the `GetTdpContents` arm (around line 88):

```rust
Event::GetReferences(e) => {
    Some(format!("[{src}] Get references: {}", e.paper))
}
```

- [ ] **Step 4: Add Telegram format test**

In the telegram tests module, add:

```rust
#[test]
fn format_get_references() {
    let listener = make_listener();
    let event = Event::GetReferences(GetReferencesEvent {
        paper: "soccer_smallsize__2024__RoboTeam".into(),
    });

    let msg = listener
        .format_message(&EventSource::Web, &event)
        .unwrap();
    assert!(msg.contains("Get references"));
    assert!(msg.contains("soccer_smallsize__2024__RoboTeam"));
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p event_processing`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add event_processing/src/lib.rs event_processing/src/listeners/telegram.rs
git commit -m "feat: add GetReferences event variant"
```

---

### Task 4: Add `get_references` API handler

**Files:**
- Create: `api/src/get_references.rs`
- Modify: `api/src/lib.rs`

- [ ] **Step 1: Write the test**

Create `api/src/get_references.rs` with the test first:

```rust
use std::sync::Arc;

use data_access::metadata::MetadataClient;
use event_processing::dispatcher::EventDispatcher;
use event_processing::{Event, EventSource, GetReferencesEvent};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetReferencesArgs {
    #[schemars(
        description = "The paper_lyt identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente')"
    )]
    pub paper: String,
}

pub async fn get_references(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetReferencesArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<String>, ApiError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;

    #[tokio::test]
    async fn test_get_references() {
        let mut mock = MockMetadataClient::new();

        let expected = vec![
            "Author A. Some paper. 2024.".to_string(),
            "Author B. Another paper. 2023.".to_string(),
        ];
        let expected_clone = expected.clone();

        mock.expect_load_references()
            .withf(|paper_lyt| paper_lyt == "soccer_smallsize__2024__RoboTeam_Twente")
            .returning(move |_| {
                let e = expected_clone.clone();
                Box::pin(std::future::ready(Ok(e)))
            });

        let client = Arc::new(mock);
        let args = GetReferencesArgs {
            paper: "soccer_smallsize__2024__RoboTeam_Twente".to_string(),
        };

        let result = get_references(client, args, &EventDispatcher::new(), EventSource::Web)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Author A. Some paper. 2024.");
        assert_eq!(result[1], "Author B. Another paper. 2023.");
    }

    #[tokio::test]
    async fn test_get_references_empty() {
        let mut mock = MockMetadataClient::new();

        mock.expect_load_references()
            .withf(|paper_lyt| paper_lyt == "soccer_smallsize__2024__SomeTeam")
            .returning(move |_| {
                Box::pin(std::future::ready(Ok(vec![])))
            });

        let client = Arc::new(mock);
        let args = GetReferencesArgs {
            paper: "soccer_smallsize__2024__SomeTeam".to_string(),
        };

        let result = get_references(client, args, &EventDispatcher::new(), EventSource::Web)
            .await
            .unwrap();

        assert!(result.is_empty());
    }
}
```

- [ ] **Step 2: Add module to `lib.rs`**

In `api/src/lib.rs`, add:

```rust
pub mod get_references;
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p api -- test_get_references`

Expected: FAIL — `todo!()` panics.

- [ ] **Step 4: Implement the handler**

Replace the `todo!()` body in `get_references`:

```rust
pub async fn get_references(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetReferencesArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<String>, ApiError> {
    let references = metadata_client
        .load_references(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    dispatcher.dispatch(
        source,
        Event::GetReferences(GetReferencesEvent {
            paper: args.paper.clone(),
        }),
    );

    Ok(references)
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p api -- test_get_references`

Expected: Both PASS.

- [ ] **Step 6: Commit**

```bash
git add api/src/get_references.rs api/src/lib.rs
git commit -m "feat: add get_references API handler"
```

---

### Task 5: Wire up MCP tool and web route

**Files:**
- Modify: `mcp/src/server.rs`
- Create: `web/src/routes/references.rs`
- Modify: `web/src/routes/mod.rs`

- [ ] **Step 1: Add MCP tool**

In `mcp/src/server.rs`, add `get_references` to the import line (line 2):

```rust
use api::{get_abstract, get_league_info, get_paper_info, get_references, get_section, get_table_of_contents, get_tdp_contents, get_team_info, list_leagues, list_papers, list_teams, list_years, paper_filter, search, suggestion};
```

Add the tool method to the `AppServer` impl, after the `get_abstract` tool (around line 200):

```rust
#[tool(
    description = "Get the references/bibliography of a paper. Requires the paper paper_lyt identifier. Returns a list of cited works — useful for finding related papers and understanding what prior work a team builds on."
)]
pub async fn get_references(
    &self,
    Parameters(args): Parameters<get_references::GetReferencesArgs>,
) -> Result<CallToolResult, McpError> {
    match get_references::get_references(self.state.metadata_client.clone(), args, &self.state.dispatcher, event_processing::EventSource::Mcp).await {
        Ok(refs) => {
            let text = if refs.is_empty() {
                "No references found for this paper.".to_string()
            } else {
                refs.iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {}", i + 1, r))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(CallToolResult::success(vec![Content::text(text)]))
        },
        Err(e) => Err(McpError::internal_error(e.to_string(), None)),
    }
}
```

- [ ] **Step 2: Create web route handler**

Create `web/src/routes/references.rs`:

```rust
use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_references_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let args = api::get_references::GetReferencesArgs { paper: paper_lyt };
    let result = api::get_references::get_references(
        state.metadata_client.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
```

- [ ] **Step 3: Register the route**

In `web/src/routes/mod.rs`, add the module declaration (after `mod abstract_text;`, around line 1):

```rust
mod references;
```

Add the route (after the `/api/papers/{id}/abstract` line, around line 41):

```rust
.route("/api/papers/{id}/references", get(references::get_references_handler))
```

- [ ] **Step 4: Verify everything compiles**

Run: `cargo build`

Expected: Compiles without errors.

- [ ] **Step 5: Run all tests**

Run: `cargo test`

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add mcp/src/server.rs web/src/routes/references.rs web/src/routes/mod.rs
git commit -m "feat: wire up get_references MCP tool and web route

MCP: get_references tool returns numbered reference list.
Web: GET /api/papers/{id}/references returns Vec<String>."
```

---

### Task 6: Final verification

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`

Expected: All tests pass, no warnings related to our changes.

- [ ] **Step 2: Verify the compile is clean**

Run: `cargo build 2>&1 | grep -i warning || echo "No warnings"`

Expected: No warnings related to our new code.
