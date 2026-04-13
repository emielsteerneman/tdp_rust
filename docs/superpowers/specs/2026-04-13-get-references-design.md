# Get References Feature

## Summary

Add a `get_references` API handler that returns the bibliography/references section of a TDP. This requires:
1. Fixing the markdown parser to handle the actual reference formats in the corpus
2. Storing references in SQLite during ingestion
3. Adding a `load_references` method to `MetadataClient`
4. Creating the API handler, event variant, and MCP/web wiring

## Context

- `MarkdownTDP` already has a `references: Vec<String>` field
- The parser currently only matches `* ` prefixed lines — a format used by exactly 1 paper (now fixed)
- 1875 papers use `N. ` format, 191 use `[N]` format, 32 are empty, 26 are edge cases (deferred)
- `store_paper` ignores `references` entirely — they're parsed then discarded
- The RoboCanes paper (the sole `*` format paper) has been normalized to numbered format

## Changes by Crate

### 1. `data_processing` — Fix parser (`markdown_parser.rs`)

**Current** (line 391): Only matches `* ` prefix.

**New**: Match numbered references in two formats:
- `N. text` — e.g. `1. Author, Title...`
- `[N] text` — e.g. `[1] Author, Title...`

Strip the number prefix (`1. ` or `[1] `) and store only the citation text — the `seq` field captures ordering. Non-empty lines in the References section that don't match either pattern are skipped.

Update the existing parser test to use numbered format. Add a test for `[N]` bracket format.

### 2. `data_access` — Store and load references

**Schema** — new `reference` table:
```sql
CREATE TABLE IF NOT EXISTS reference (
    paper_lyt TEXT NOT NULL,
    seq INTEGER NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt),
    UNIQUE(paper_lyt, seq)
)
```

**`store_paper`**: After inserting authors and toc_entries, insert references with sequential `seq` values. Add `DELETE FROM reference WHERE paper_lyt = ?1` to the upsert cleanup block.

**`MetadataClient` trait** — new method:
```rust
fn load_references<'a>(
    &'a self,
    paper_lyt: String,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>, MetadataClientError>> + Send + 'a>>;
```

Returns `Vec<String>` — just the reference texts in order. Empty vec if the paper has no references.

### 3. `event_processing` — New event variant

```rust
pub struct GetReferencesEvent {
    pub paper: String,
}
```

Add `GetReferences(GetReferencesEvent)` to the `Event` enum. Wire into:
- `event_name()` → `"get_references"`
- Telegram listener format → `"[{src}] Get references: {paper}"`
- SQLite activity listener (follows existing pattern)
- Add to test vectors

### 4. `api` — New handler (`get_references.rs`)

Mirrors `get_abstract`:

```rust
pub struct GetReferencesArgs {
    pub paper: String,  // paper_lyt identifier
}

pub async fn get_references(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetReferencesArgs,
    dispatcher: &EventDispatcher,
    source: EventSource,
) -> Result<Vec<String>, ApiError>
```

Returns `Vec<String>` of reference texts. Re-export from `lib.rs`.

### 5. `mcp` — New tool (`server.rs`)

```rust
#[tool(
    description = "Get the references/bibliography of a paper. Requires the paper paper_lyt identifier. Returns a list of cited works."
)]
pub async fn get_references(
    &self,
    Parameters(args): Parameters<get_references::GetReferencesArgs>,
) -> Result<CallToolResult, McpError>
```

Format as numbered list in the text response.

### 6. `web` — New route

- New file: `web/src/routes/references.rs`
- Route: `GET /api/papers/{id}/references`
- Returns `Json<ApiResponse<Vec<String>>>`

## What Requires a Reindex

Storing references in SQLite requires running `initialize` again. The `reference` table is new, so existing DBs will get the table on startup (via `CREATE TABLE IF NOT EXISTS`), but it will be empty until papers are re-ingested.

## Out of Scope

- Parsing the 26 edge-case papers with non-standard reference formats
- Structured reference parsing (extracting author, title, year, URL as separate fields)
- Cross-paper reference linking
- Frontend UI for displaying references
