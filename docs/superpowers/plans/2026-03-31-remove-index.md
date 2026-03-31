# Remove Index from TDP Naming — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the `index` field from the TDP naming system, changing from 4-part `league__year__team__index` to 3-part `league__year__team`, renamed from `lyti` to `paper_lyt` throughout.

**Architecture:** Bottom-up refactor: data_structures → data_access → data_processing → api → mcp/web → frontend → tools → docs. Each task modifies one crate/layer, tests pass after each task. Full reindex required after merge.

**Tech Stack:** Rust, SQLite, Qdrant, SvelteKit/TypeScript, Axum, rmcp

---

### Task 1: data_structures — TDPName (remove index field)

**Files:**
- Modify: `data_structures/src/file/tdp_name.rs`

- [ ] **Step 1: Update TDPParseError — remove Index variant**

```rust
#[derive(thiserror::Error, Debug)]
pub enum TDPParseError {
    #[error("expected 3 fields separated by '__', got {0}")]
    BadFieldCount(usize),
    #[error("Could not parse league: {0}")]
    League(#[from] LeagueParseError),
    #[error("invalid team name: {0}")]
    Team(String),
    #[error("invalid year: {0}")]
    Year(String),
    #[error("missing file stem in path")]
    NoFileStem,
}
```

- [ ] **Step 2: Update TDPName struct — remove index field**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TDPName {
    pub league: League,
    pub team_name: TeamName,
    pub year: u32,
}
```

- [ ] **Step 3: Update constructor — remove index parameter**

```rust
impl TDPName {
    pub const PDF_EXT: &'static str = ".pdf";
    pub const HTML_EXT: &'static str = ".html";

    pub fn new(league: League, year: u32, team_name: TeamName) -> Self {
        Self {
            league,
            team_name,
            year,
        }
    }

    pub fn get_paper_lyt(&self) -> String {
        format!(
            "{}__{}__{}",
            self.league.name(), self.year, self.team_name.name
        )
    }
}
```

- [ ] **Step 4: Update TryFrom — parse 3 fields instead of 4**

```rust
impl TryFrom<&str> for TDPName {
    type Error = TDPParseError;

    fn try_from(string: &str) -> Result<Self, TDPParseError> {
        let base = match string.rsplit_once('.') {
            Some((stem, _ext)) => stem,
            None => &string,
        };

        let parts: Vec<&str> = base.split("__").collect();
        if parts.len() != 3 {
            return Err(TDPParseError::BadFieldCount(parts.len()));
        }

        let l = parts[0];
        let y = parts[1];
        let t = parts[2];

        let league: League = l.try_into()?;
        let year: u32 = y.parse().map_err(|_| TDPParseError::Year(y.to_string()))?;
        let team_name: TeamName = TeamName::new(t);

        Ok(Self::new(league, year, team_name))
    }
}
```

- [ ] **Step 5: Update tests**

```rust
#[cfg(test)]
mod tests {
    use crate::file::{League, TDPName};

    #[test]
    pub fn test_basic() {
        let filename = "soccer_smallsize__2019__RoboTeam_Twente.pdf";
        let tdp_name: TDPName = filename.try_into().unwrap();

        assert_eq!(tdp_name.league, League::SoccerSmallSize);
        assert_eq!(tdp_name.league.name_pretty(), "Soccer SmallSize");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name, "RoboTeam_Twente");
        assert_eq!(tdp_name.team_name.name_pretty, "RoboTeam Twente");
    }

    #[test]
    pub fn test_deserialize() {
        let json = r#"{"league": "industrial_atwork", "team_name": {"name": "Carologistics", "name_pretty": "Carologistics"}, "year": 2019}"#;

        let tdp_name: TDPName = serde_json::from_str(json).unwrap();

        assert_eq!(tdp_name.league, League::IndustrialAtwork);
        assert_eq!(tdp_name.league.name_pretty(), "Industrial @Work");
        assert_eq!(tdp_name.year, 2019);
        assert_eq!(tdp_name.team_name.name, "Carologistics");
        assert_eq!(tdp_name.team_name.name_pretty, "Carologistics");
    }
}
```

- [ ] **Step 6: Verify data_structures compiles**

Run: `cargo check -p data_structures`
Expected: compiler errors in downstream crates only (they still reference `.index` and `get_filename()`)

Note: Do NOT run `cargo check` (whole workspace) yet — downstream crates will break until we fix them.

---

### Task 2: data_structures — Chunk, Filter, SearchResultChunk, SectionResult (rename fields)

**Files:**
- Modify: `data_structures/src/intermediate/chunk.rs`
- Modify: `data_structures/src/intermediate/search.rs`
- Modify: `data_structures/src/intermediate/navigation.rs`
- Modify: `data_structures/src/filter.rs`

- [ ] **Step 1: Update Chunk — rename field**

In `data_structures/src/intermediate/chunk.rs`, change:

```rust
    pub league_year_team_idx: String,
```
to:
```rust
    pub paper_lyt: String,
```

And in `to_uuid()`:
```rust
        let s = format!(
            "{}__{}__{}",
            self.paper_lyt.clone(),
            self.content_seq,
            self.chunk_seq
        );
```

And in the `Debug` impl — no change needed (it doesn't print this field).

- [ ] **Step 2: Update SearchResultChunk — rename field**

In `data_structures/src/intermediate/search.rs`, change:

```rust
pub struct SearchResultChunk {
    pub paper_lyt: String,
    // ... rest unchanged
}

impl From<Chunk> for SearchResultChunk {
    fn from(chunk: Chunk) -> Self {
        Self {
            paper_lyt: chunk.paper_lyt,
            // ... rest unchanged
        }
    }
}
```

- [ ] **Step 3: Update SectionResult — rename field**

In `data_structures/src/intermediate/navigation.rs`, change:

```rust
pub struct SectionResult {
    pub paper_lyt: String,
    pub breadcrumbs: Vec<BreadcrumbEntry>,
    pub items: Vec<ContentItem>,
}
```

- [ ] **Step 4: Update Filter — rename field and methods**

In `data_structures/src/filter.rs`:

```rust
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Filter {
    #[schemars(description = "An optional list of team names on which to filter results")]
    pub teams: Option<HashSet<String>>,
    #[schemars(description = "An optional list of leagues on which to filter results")]
    pub leagues: Option<HashSet<League>>,
    #[schemars(description = "An optional list of years on which to filter results")]
    pub years: Option<HashSet<u32>>,
    #[schemars(
        description = "An optional list of paper_lyt identifiers on which to filter results"
    )]
    pub paper_lyts: Option<HashSet<String>>,
    #[schemars(
        description = "An optional list of content types on which to filter results (text, table, image)"
    )]
    pub content_types: Option<HashSet<String>>,
}

impl Filter {
    // ... add_team, add_league, add_year, add_content_type unchanged ...

    pub fn add_tdp(&mut self, tdp_name: TDPName) {
        self.add_paper_lyt(tdp_name.get_paper_lyt());
    }

    pub fn add_paper_lyt(&mut self, paper_lyt: String) {
        let paper_lyts = self.paper_lyts.get_or_insert_with(HashSet::new);
        paper_lyts.insert(paper_lyt);
    }

    pub fn matches_tdp_name(&self, tdp_name: &TDPName) -> bool {
        if let Some(teams) = &self.teams {
            if !teams.contains(&tdp_name.team_name.name) {
                return false;
            }
        }
        if let Some(leagues) = &self.leagues {
            if !leagues.contains(&tdp_name.league) {
                return false;
            }
        }
        if let Some(years) = &self.years {
            if !years.contains(&tdp_name.year) {
                return false;
            }
        }
        if let Some(paper_lyts) = &self.paper_lyts {
            if !paper_lyts.contains(&tdp_name.get_paper_lyt()) {
                return false;
            }
        }
        true
    }
}
```

- [ ] **Step 5: Verify data_structures compiles**

Run: `cargo check -p data_structures`
Expected: PASS (data_structures is self-contained now)

---

### Task 3: data_access — Qdrant client (rename KEY_LYTI, update payload)

**Files:**
- Modify: `data_access/src/vector/qdrant_client.rs`

- [ ] **Step 1: Rename constant and update payload storage**

Change:
```rust
    const KEY_LYTI: &'static str = "lyti";
```
to:
```rust
    const KEY_PAPER_LYT: &'static str = "paper_lyt";
```

- [ ] **Step 2: Update store_chunk — use new field name**

In `store_chunk`, change:
```rust
        payload.insert(Self::KEY_LYTI.into(), chunk.league_year_team_idx.into());
```
to:
```rust
        payload.insert(Self::KEY_PAPER_LYT.into(), chunk.paper_lyt.into());
```

- [ ] **Step 3: Update search_chunks filter — rename field access**

In `search_chunks`, change:
```rust
            if let Some(indexes) = f.league_year_team_indexes {
                if !indexes.is_empty() {
                    info!("Adding lity filter {:?}", indexes);
                    conditions.push(Condition::matches(
                        Self::KEY_LYTI,
                        indexes.into_iter().collect::<Vec<String>>(),
                    ));
                }
            }
```
to:
```rust
            if let Some(paper_lyts) = f.paper_lyts {
                if !paper_lyts.is_empty() {
                    info!("Adding paper_lyt filter {:?}", paper_lyts);
                    conditions.push(Condition::matches(
                        Self::KEY_PAPER_LYT,
                        paper_lyts.into_iter().collect::<Vec<String>>(),
                    ));
                }
            }
```

- [ ] **Step 4: Update IntoChunk for HashMap — rename field**

In the `IntoChunk` impl for `HashMap<String, Value>`:

Change:
```rust
        let league_year_team_idx = from_payload_get_string(&self, QdrantClient::KEY_LYTI)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_LYTI.to_string()))?;
```
to:
```rust
        let paper_lyt = from_payload_get_string(&self, QdrantClient::KEY_PAPER_LYT)
            .ok_or_else(|| VectorClientError::FieldMissing(QdrantClient::KEY_PAPER_LYT.to_string()))?;
```

And in the `Ok(Chunk { ... })` at the bottom:
```rust
            paper_lyt,
```

Also remove the `// League Year Team Index` comment and replace with `// Paper identifier`.

- [ ] **Step 5: Update tests — rename all league_year_team_idx references**

In all test chunks, change `league_year_team_idx: "soccer_smallsize__1998__test_team__0".to_string()` to `paper_lyt: "soccer_smallsize__1998__test_team".to_string()` (and similarly for all other test chunks).

Update assertions: `chunk.league_year_team_idx` → `chunk.paper_lyt`.

Remove `chunk_2_2` from `test_store_and_retrieve_with_filter` (it was testing `__1` which no longer exists). Update the league/year/team filter test to expect 1 result instead of 2 for `chunk_2_1`.

- [ ] **Step 6: Verify data_access compiles**

Run: `cargo check -p data_access`
Expected: PASS

---

### Task 4: data_access — SQLite client (remove idx column, rename lyti)

**Files:**
- Modify: `data_access/src/metadata/sqlite_client.rs`
- Modify: `data_access/src/metadata/mod.rs`

- [ ] **Step 1: Update paper table schema — remove idx, rename lyti**

In `ensure_database_paper_v2`:

```rust
    fn ensure_database_paper_v2(&self) {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS paper (
                paper_lyt TEXT PRIMARY KEY,
                league TEXT NOT NULL,
                year INTEGER NOT NULL,
                team TEXT NOT NULL,
                title TEXT,
                abstract_text TEXT,
                urls_json TEXT,
                raw_markdown TEXT
            )",
            [],
        )
        .expect("Failed to create table paper");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS author (
                paper_lyt TEXT NOT NULL,
                name TEXT NOT NULL,
                affiliation TEXT,
                FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt)
            )",
            [],
        )
        .expect("Failed to create table author");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS toc_entry (
                paper_lyt TEXT NOT NULL,
                content_seq INTEGER NOT NULL,
                content_type TEXT NOT NULL,
                depth INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                image_path TEXT,
                FOREIGN KEY (paper_lyt) REFERENCES paper(paper_lyt),
                UNIQUE(paper_lyt, content_seq)
            )",
            [],
        )
        .expect("Failed to create table toc_entry");

        conn.execute("CREATE INDEX IF NOT EXISTS paper_league ON paper (league)", [])
            .expect("Failed to create index on paper (league)");
        conn.execute("CREATE INDEX IF NOT EXISTS paper_year ON paper (year)", [])
            .expect("Failed to create index on paper (year)");
        conn.execute("CREATE INDEX IF NOT EXISTS paper_team ON paper (team)", [])
            .expect("Failed to create index on paper (team)");
    }
```

- [ ] **Step 2: Update store_paper — remove idx, use paper_lyt**

In `store_paper`:

```rust
                let paper_lyt = tdp.name.get_paper_lyt();
                let league = tdp.name.league.name();
                let year = tdp.name.year;
                let team = &tdp.name.team_name.name_pretty;
                let title = &tdp.front_matter.title;
                let abstract_text = tdp.front_matter.abstract_text.as_deref();
                let urls_json = serde_json::to_string(&tdp.front_matter.urls)
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                let raw_markdown = &tdp.raw_markdown;

                // ... transaction setup unchanged ...

                    tx.execute("DELETE FROM toc_entry WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    tx.execute("DELETE FROM author WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    tx.execute("DELETE FROM paper WHERE paper_lyt = ?1", params![paper_lyt])
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    tx.execute(
                        "INSERT INTO paper (paper_lyt, league, year, team, title, abstract_text, urls_json, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![paper_lyt, league, year, team, title, abstract_text, urls_json, raw_markdown],
                    )
                    .map_err(|e| MetadataClientError::Internal(e.to_string()))?;

                    let mut author_stmt = tx
                        .prepare("INSERT INTO author (paper_lyt, name, affiliation) VALUES (?1, ?2, ?3)")
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    // ... author insert loop uses paper_lyt ...

                    let mut toc_stmt = tx
                        .prepare("INSERT INTO toc_entry (paper_lyt, content_seq, content_type, depth, title, body, image_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)")
                        .map_err(|e| MetadataClientError::Internal(e.to_string()))?;
                    // ... toc insert loop uses paper_lyt ...
```

- [ ] **Step 3: Update all query methods — replace lyti with paper_lyt**

Rename the `lyti` parameter in all trait methods and their implementations:
- `load_toc(lyti: String)` → `load_toc(paper_lyt: String)`
- `load_content_item(lyti: String, ...)` → `load_content_item(paper_lyt: String, ...)`
- `load_content_items_range(lyti: String, ...)` → `load_content_items_range(paper_lyt: String, ...)`
- `load_paper_abstract(lyti: String)` → `load_paper_abstract(paper_lyt: String)`
- `load_paper_info(lyti: String)` → `load_paper_info(paper_lyt: String)`

In every SQL query, replace `WHERE lyti = ?1` with `WHERE paper_lyt = ?1`. Replace all error messages mentioning "lyti" with "paper_lyt".

Update `load_tdps` — uses `SELECT lyti FROM paper` → `SELECT paper_lyt FROM paper`.

Update `get_tdp_markdown` — uses `tdp_name.get_filename()` → `tdp_name.get_paper_lyt()`, `WHERE lyti = ?1` → `WHERE paper_lyt = ?1`.

- [ ] **Step 4: Update MetadataClient trait in mod.rs — rename parameters**

In `data_access/src/metadata/mod.rs`, rename all `lyti: String` parameters to `paper_lyt: String`:

```rust
    fn load_toc<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TocEntry>, MetadataClientError>> + Send + 'a>>;

    fn load_content_item<'a>(
        &'a self,
        paper_lyt: String,
        content_seq: u32,
    ) -> Pin<Box<dyn Future<Output = Result<ContentItem, MetadataClientError>> + Send + 'a>>;

    fn load_paper_abstract<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, MetadataClientError>> + Send + 'a>>;

    fn load_content_items_range<'a>(
        &'a self,
        paper_lyt: String,
        start_seq: u32,
        end_seq_exclusive: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ContentItem>, MetadataClientError>> + Send + 'a>>;

    fn load_paper_info<'a>(
        &'a self,
        paper_lyt: String,
    ) -> Pin<Box<dyn Future<Output = Result<PaperInfo, MetadataClientError>> + Send + 'a>>;
```

- [ ] **Step 5: Update tests — remove idx, use 3-part names**

In `test_load_teams_and_leagues`, update all INSERT statements:
```sql
INSERT INTO paper (paper_lyt, league, year, team, raw_markdown) VALUES (?1, ?2, ?3, ?4, ?5)
```
With values like `"soccer_smallsize__2019__RoboTeam_Twente"` (no `__1`).

In `test_store_and_load_paper`, update:
```rust
        let name = data_structures::file::TDPName::new(league.clone(), 2024, team.clone());
        let paper_lyt = name.get_paper_lyt();
```
Replace all `lyti` variable references with `paper_lyt`.

- [ ] **Step 6: Verify data_access compiles**

Run: `cargo check -p data_access`
Expected: PASS

---

### Task 5: data_processing — content_chunker and search (rename fields)

**Files:**
- Modify: `data_processing/src/content_chunker.rs`
- Modify: `data_processing/src/search.rs`

- [ ] **Step 1: Update content_chunker — rename field in all chunk constructors**

Replace all 3 occurrences of:
```rust
                    league_year_team_idx: tdp.name.get_filename(),
```
with:
```rust
                    paper_lyt: tdp.name.get_paper_lyt(),
```

- [ ] **Step 2: Update content_chunker test — use 3-part name**

In `make_tdp`:
```rust
    fn make_tdp(content_items: Vec<ContentItem>) -> MarkdownTDP {
        let name: TDPName = "soccer_smallsize__2024__TestTeam".try_into().unwrap();
        MarkdownTDP {
            name,
            front_matter: FrontMatter::default(),
            content_items,
            references: vec![],
            raw_markdown: String::new(),
        }
    }
```

- [ ] **Step 3: Update search.rs — rename lyti variables**

In `data_processing/src/search.rs`, rename:
- `unique_lytis` → `unique_paper_lyts`
- `chunk.league_year_team_idx` → `chunk.paper_lyt`
- `lyti` loop variable → `paper_lyt`

```rust
        let unique_paper_lyts: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            result.chunks.iter().filter_map(|chunk| {
                if seen.insert(chunk.paper_lyt.clone()) {
                    Some(chunk.paper_lyt.clone())
                } else {
                    None
                }
            }).collect()
        };

        for paper_lyt in unique_paper_lyts {
            match self.metadata_client.load_toc(paper_lyt.clone()).await {
                Ok(toc) => {
                    toc_cache.insert(paper_lyt, toc);
                }
                Err(e) => {
                    warn!("Failed to load ToC for {}: {}", paper_lyt, e);
                }
            }
        }
```

And the breadcrumb lookup:
```rust
                    .get(&chunk.paper_lyt)
```

- [ ] **Step 4: Verify data_processing compiles**

Run: `cargo check -p data_processing`
Expected: PASS

---

### Task 6: api — Search handler and all paper handlers (rename params)

**Files:**
- Modify: `api/src/search.rs`
- Modify: `api/src/get_abstract.rs`
- Modify: `api/src/get_paper_info.rs`
- Modify: `api/src/get_table_of_contents.rs`
- Modify: `api/src/get_section.rs`
- Modify: `api/src/get_table.rs`
- Modify: `api/src/get_image.rs`
- Modify: `api/src/get_paragraph.rs`

- [ ] **Step 1: Update search.rs — rename lyti_filter**

Change:
```rust
    #[schemars(
        description = "Optional comma-separated filter for specific papers by their league__year__team__index identifier, e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0'. Rarely needed — prefer league/year/team filters."
    )]
    pub lyti_filter: Option<String>,
```
to:
```rust
    #[schemars(
        description = "Optional comma-separated filter for specific papers by their paper_lyt identifier, e.g. 'soccer_smallsize__2024__RoboTeam_Twente'. Rarely needed — prefer league/year/team filters."
    )]
    pub paper_lyt_filter: Option<String>,
```

And in `to_filter()`:
```rust
        if let Some(paper_lyt_filter) = &self.paper_lyt_filter {
            for paper_lyt in paper_lyt_filter.split(",") {
                filter.add_paper_lyt(paper_lyt.trim().to_string());
            }
        }
```

- [ ] **Step 2: Update search.rs test**

Change:
```rust
            lyti_filter: Some("rescue_simulation_infrastructure__2012__UvA_Rescue__0".to_string()),
```
to:
```rust
            paper_lyt_filter: Some("rescue_simulation_infrastructure__2012__UvA_Rescue".to_string()),
```

And the assertion:
```rust
        assert!(
            filter
                .paper_lyts
                .as_ref()
                .unwrap()
                .contains("rescue_simulation_infrastructure__2012__UvA_Rescue")
        );
```

- [ ] **Step 3: Update all paper handler descriptions — 6 files**

In each of these files, update the `#[schemars(description)]` on the `paper` field:

`get_abstract.rs`, `get_paper_info.rs`, `get_table_of_contents.rs`, `get_section.rs`, `get_table.rs`, `get_image.rs`, `get_paragraph.rs`:

Change:
```rust
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
```
to:
```rust
    #[schemars(
        description = "The paper_lyt identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente')"
    )]
    pub paper: String,
```

- [ ] **Step 4: Update get_section.rs — rename SectionResult field**

In `get_section`:
```rust
    Ok(SectionResult {
        paper_lyt: args.paper,
        breadcrumbs,
        items,
    })
```

- [ ] **Step 5: Update all test lyti strings — remove __0 suffix**

In every test across all api handler files, replace `"soccer_smallsize__2024__RoboTeam_Twente__0"` with `"soccer_smallsize__2024__RoboTeam_Twente"`, and `"soccer_smallsize__2024__Test__0"` with `"soccer_smallsize__2024__Test"`.

Also update `result.lyti` assertions to `result.paper_lyt`.

- [ ] **Step 6: Verify api compiles and tests pass**

Run: `cargo test -p api`
Expected: All tests pass

---

### Task 7: api — list_papers and paper_filter (update TDPName::new calls)

**Files:**
- Modify: `api/src/list_papers.rs`
- Modify: `api/src/paper_filter.rs`

- [ ] **Step 1: Update list_papers test — remove None index parameter**

In `test_list_papers`, change all `TDPName::new` calls from:
```rust
TDPName::new(League::SoccerSmallSize, 2019, TeamName::from_pretty("RoboTeam Twente"), None)
```
to:
```rust
TDPName::new(League::SoccerSmallSize, 2019, TeamName::from_pretty("RoboTeam Twente"))
```

- [ ] **Step 2: Update paper_filter tests — remove None index parameter**

Same change in `test_papers()` helper and all test functions.

- [ ] **Step 3: Verify api tests pass**

Run: `cargo test -p api`
Expected: All tests pass

---

### Task 8: mcp — Server tool descriptions and CompactChunk (rename lyti)

**Files:**
- Modify: `mcp/src/server.rs`

- [ ] **Step 1: Rename CompactChunk field**

```rust
#[derive(Serialize)]
struct CompactChunk {
    paper_lyt: String,
    content_seq: u32,
    title: String,
    content_type: String,
    score: f32,
    text: String,
    section_path: Vec<BreadcrumbEntry>,
}
```

- [ ] **Step 2: Update search tool — use new field name**

In the search tool handler:
```rust
                    results: result.chunks.into_iter().map(|c| CompactChunk {
                        paper_lyt: c.paper_lyt,
                        // ... rest unchanged
                    }).collect(),
```

- [ ] **Step 3: Update list_papers tool — description and get_paper_lyt**

Change description:
```rust
    #[tool(
        description = "List papers in the database with optional filters. Returns paper_lyt identifiers (e.g. 'soccer_smallsize__2024__RoboTeam_Twente') for matching TDPs. Always use at least one filter to avoid returning all 2000+ papers. Examples: filter by league='Soccer SmallSize' and year=2024 to see that year's teams, or team='TIGERs Mannheim' to see all their papers."
    )]
```

And the map:
```rust
                let paper_lyts: Vec<String> = papers.iter().map(|p| p.get_paper_lyt()).collect();
                match serde_json::to_string_pretty(&paper_lyts) {
```

- [ ] **Step 4: Update tool descriptions — remove __0 from examples**

Update these tool descriptions:
- `get_table_of_contents`: change lyti example to `'soccer_smallsize__2024__RoboTeam_Twente'`
- `get_section`: change "paper lyti" to "paper paper_lyt"
- `get_abstract`: change "paper lyti identifier" to "paper paper_lyt identifier"
- `get_paper_info`: change "paper lyti identifier" to "paper paper_lyt identifier"

- [ ] **Step 5: Update server instructions**

Change line 341 area:
```rust
A **paper_lyt** (League-Year-Team) is the unique paper identifier used across all tools, e.g. `soccer_smallsize__2024__RoboTeam_Twente`.
```

- [ ] **Step 6: Verify mcp compiles**

Run: `cargo check -p mcp`
Expected: PASS

---

### Task 9: web — Routes and TDP file serving (rename params)

**Files:**
- Modify: `web/src/routes/papers.rs`
- Modify: `web/src/routes/tdps.rs`
- Modify: `web/src/routes/search.rs` (no changes needed, uses Query<SearchArgs>)

- [ ] **Step 1: Update papers.rs — rename lyti variables**

In `paper_open_handler` and `pdf_open_handler`, rename `Path(lyti)` → `Path(paper_lyt)`:

```rust
pub async fn paper_open_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
    headers: HeaderMap,
) -> StatusCode {
    // ... referrer extraction unchanged ...
    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::PaperOpen(event_processing::PaperOpenEvent {
            paper_id: paper_lyt,
            referrer,
        }),
    );
    StatusCode::NO_CONTENT
}

pub async fn pdf_open_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
) -> StatusCode {
    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::PdfOpen(event_processing::PdfOpenEvent {
            paper_id: paper_lyt,
        }),
    );
    StatusCode::NO_CONTENT
}
```

- [ ] **Step 2: Update tdps.rs — rename lyti variables and comments**

Rename all `lyti_str`, `lyti`, `lyti_for_parse`, `lyti_base` variables to `paper_lyt_str`, `paper_lyt`, `paper_lyt_for_parse`, `paper_lyt_base`. Update comments:

```rust
    // The path is either:
    //   {paper_lyt}.md          — the markdown file itself
    //   {paper_lyt}/{subpath}   — a file inside the paper's image folder
    let (paper_lyt_str, subpath) = if let Some((lyt, sub)) = path.split_once('/') {
        (lyt, Some(sub))
    } else {
        (path.as_str(), None)
    };

    let paper_lyt_for_parse = if paper_lyt_str.ends_with(".md") {
        &paper_lyt_str[..paper_lyt_str.len() - 3]
    } else {
        paper_lyt_str
    };

    let tdp_name: TDPName = TDPName::try_from(paper_lyt_for_parse).map_err(|_| StatusCode::BAD_REQUEST)?;
    // ... league path construction unchanged ...

    let paper_lyt_base = paper_lyt_for_parse;

    let file_path = match subpath {
        None => {
            // Markdown file: {league_path}/{paper_lyt}.md
            league_path.join(format!("{}.md", paper_lyt_base))
        }
        Some(sub) => {
            // Image/asset inside paper folder: {league_path}/{paper_lyt}/{subpath}
            league_path.join(paper_lyt_base).join(sub)
        }
    };
```

- [ ] **Step 3: Verify web compiles**

Run: `cargo check -p web`
Expected: PASS

---

### Task 10: Full Rust workspace — compile and test

**Files:** None (verification only)

- [ ] **Step 1: Run full workspace check**

Run: `cargo check`
Expected: PASS — all crates compile

- [ ] **Step 2: Run all tests (excluding integration tests that need Docker)**

Run: `cargo test -- --skip test_store_and_retrieve --skip test_create_client --skip test_analyze`
Expected: All non-integration tests pass

- [ ] **Step 3: Commit Rust changes**

```bash
git add data_structures/ data_access/ data_processing/ api/ mcp/ web/ tools/
git commit -m "refactor: remove index from TDP naming, rename lyti to paper_lyt

The 4-part league__year__team__index format is now 3-part
league__year__team. The index field served no practical purpose
(only 28 of 2000+ papers had index > 0, mostly duplicates).

- Remove TDPName.index field, update parser to expect 3 parts
- Rename get_filename() to get_paper_lyt()
- Rename league_year_team_idx to paper_lyt throughout
- Rename lyti to paper_lyt in SQLite schema and Qdrant payload
- Update all API handler descriptions and MCP tool descriptions
- Remove idx column from SQLite paper table

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 11: Frontend — TypeScript types and components

**Files:**
- Modify: `frontend/src/lib/types.ts`
- Modify: `frontend/src/lib/api.ts`
- Modify: `frontend/src/lib/markdown.ts`
- Modify: `frontend/src/lib/components/PaperCard.svelte`
- Modify: `frontend/src/routes/paper/[id]/+page.ts`
- Modify: `frontend/src/routes/paper/[id]/+page.svelte`
- Modify: `frontend/src/routes/(browse)/search/+page.svelte`

- [ ] **Step 1: Update types.ts**

```typescript
export interface TDPName {
	league: League;
	team_name: TeamName;
	year: number;
}

export interface SearchResultChunk {
	paper_lyt: string;
	// ... rest unchanged
}

export interface Filter {
	leagues?: string[];
	years?: number[];
	teams?: string[];
	paper_lyts?: string[];
}

export interface SearchParams {
	query: string;
	limit?: number;
	league_filter?: string;
	year_filter?: string;
	team_filter?: string;
	paper_lyt_filter?: string;
	content_type_filter?: string;
	search_type?: EmbedType;
}
```

- [ ] **Step 2: Update api.ts — rename params**

Change:
```typescript
	if (params.lyti_filter) {
		searchParams.append('lyti_filter', params.lyti_filter);
	}
```
to:
```typescript
	if (params.paper_lyt_filter) {
		searchParams.append('paper_lyt_filter', params.paper_lyt_filter);
	}
```

And rename function parameter:
```typescript
export async function getPaperInfo(paper_lyt: string, fetchFn?: FetchFn): Promise<PaperInfo> {
	return fetchApi<PaperInfo>(`/papers/${encodeURIComponent(paper_lyt)}/info`, fetchFn);
}
```

- [ ] **Step 3: Update markdown.ts — rename parameter**

```typescript
export function preprocessMarkdown(raw: string, paper_lyt: string): string {
```

And the image reference at line 284:
```typescript
          output.push(`<img src="/tdps/${paper_lyt}/${filename}" alt="${alt}" />`);
```

- [ ] **Step 4: Update PaperCard.svelte — simplify paperId, remove index badge**

```svelte
<script lang="ts">
	import type { TDPName } from '$lib/types';

	interface Props {
		paper: TDPName;
	}

	let { paper }: Props = $props();

	let paperId = $derived(
		`${paper.league.name}__${paper.year}__${paper.team_name.name}`
	);
</script>

<a
	href="/paper/{paperId}"
	target="_blank"
	class="inline-block px-3 py-1 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 hover:border-blue-300 dark:hover:border-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors"
>
	{paper.team_name.name_pretty}
</a>
```

- [ ] **Step 5: Update paper page loader — rename lyti to paper_lyt**

In `frontend/src/routes/paper/[id]/+page.ts`:

```typescript
export const load: PageLoad = async ({ params, fetch }) => {
  const paper_lyt = params.id;

  // Parse paper_lyt: league__year__team
  const parts = paper_lyt.split('__');
  const leagueMachine = parts[0] ?? '';
  const year = parseInt(parts[1], 10) || 0;
  const teamPrettyName = parts[2]?.replace(/_/g, ' ') ?? '';

  const [rawMarkdown, paperInfo, teamEntries] = await Promise.all([
    fetch(`/tdps/${paper_lyt}.md`).then((r) => {
      if (!r.ok) throw error(r.status, 'Paper not found');
      return r.text();
    }),
    getPaperInfo(paper_lyt, fetch).catch((): PaperInfo | null => null),
    getTeamInfo(teamPrettyName, fetch).catch((): TeamMetadataEntry[] => [])
  ]);

  fetch(`/api/papers/${encodeURIComponent(paper_lyt)}/open`, { method: 'POST' }).catch(() => {});

  return {
    rawMarkdown,
    paper_lyt,
    paperInfo,
    teamEntries,
    teamPrettyName,
    leagueMachine,
    year
  };
};
```

- [ ] **Step 6: Update paper page component — use paper_lyt**

In `frontend/src/routes/paper/[id]/+page.svelte`:

```svelte
	const processed = $derived(preprocessMarkdown(data.rawMarkdown, data.paper_lyt));
```

And:
```svelte
						href="/pdfs/{data.paper_lyt}.pdf"
```

And:
```svelte
							fetch(`/api/papers/${encodeURIComponent(data.paper_lyt)}/pdf-open`, { method: 'POST' }).catch(() => {});
```

- [ ] **Step 7: Update search results page — rename field**

In `frontend/src/routes/(browse)/search/+page.svelte`:

```typescript
			const paperId = chunk.paper_lyt;
```

- [ ] **Step 8: Verify frontend builds**

Run: `cd frontend && npm run build`
Expected: Build succeeds with no TypeScript errors

- [ ] **Step 9: Commit frontend changes**

```bash
git add frontend/
git commit -m "refactor: update frontend for paper_lyt naming (remove index)

- Remove index field from TDPName type
- Rename league_year_team_idx to paper_lyt
- Rename lyti_filter to paper_lyt_filter
- Simplify PaperCard (remove index badge)
- Update all URL constructions to 3-part format

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 12: Tools — Update CLI binaries (rename lyti_filter, get_filename)

**Files:**
- Modify: `tools/src/bin/smoke_test.rs`
- Modify: `tools/src/bin/search_by_sentence.rs`
- Modify: `tools/src/bin/coverage.rs`

- [ ] **Step 1: Update smoke_test.rs — rename lyti_filter**

Change `lyti_filter: None` to `paper_lyt_filter: None`.

- [ ] **Step 2: Update search_by_sentence.rs — rename lyti_filter**

Change `lyti_filter: None` to `paper_lyt_filter: None`.

- [ ] **Step 3: Update coverage.rs — rename get_filename and lyti variables**

Change `.map(|t| t.get_filename())` to `.map(|t| t.get_paper_lyt())`.

Rename local `lyti` variables to `paper_lyt` in the coverage analysis functions (these are just local variable names in print loops — rename for consistency).

- [ ] **Step 4: Verify tools compile**

Run: `cargo check -p tools`
Expected: PASS

---

### Task 13: Documentation — Update CLAUDE.md, README.md, crate docs

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md`
- Modify: `data_structures/CLAUDE.md`
- Modify: `data_access/CLAUDE.md`
- Modify: `frontend/CLAUDE.md`
- Delete: `REMOVE_INDEX.md`

- [ ] **Step 1: Update root CLAUDE.md**

In Key Conventions:
```
- **TDP naming**: `{league}__{year}__{team}` (double underscore), e.g. `soccer_smallsize__2024__RoboTeam_Twente`
```

In Key Terms:
```
- **paper_lyt** — League Year Team. Canonical paper identifier: `soccer_smallsize__2024__RoboTeam_Twente`.
```

- [ ] **Step 2: Update README.md — same sections**

Same changes as CLAUDE.md (they share these sections).

- [ ] **Step 3: Update data_structures/CLAUDE.md**

Change:
```
- `TDPName` — parsed from `league__year__team__index` strings.
```
to:
```
- `TDPName` — parsed from `league__year__team` strings. `TryFrom<&str>` handles filenames with optional extensions.
```

- [ ] **Step 4: Update data_access/CLAUDE.md — if it references lyti**

No direct lyti references in current content. No change needed.

- [ ] **Step 5: Update frontend/CLAUDE.md — if it references lyti**

No direct lyti references in current content. No change needed.

- [ ] **Step 6: Delete REMOVE_INDEX.md**

```bash
rm REMOVE_INDEX.md
```

- [ ] **Step 7: Commit documentation changes**

```bash
git add CLAUDE.md README.md data_structures/CLAUDE.md REMOVE_INDEX.md
git commit -m "docs: update naming convention from lyti to paper_lyt

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```
