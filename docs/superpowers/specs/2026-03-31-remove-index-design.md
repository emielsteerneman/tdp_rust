# Remove Index from TDP Naming

**Date:** 2026-03-31
**Status:** Approved

## Problem

The lyti format `{league}__{year}__{team}__{index}` includes an index to support teams with multiple papers per year. In practice, only 28 out of ~2000 papers have index > 0, and most are duplicates or shorter versions of the `__0` paper. The index complicates URL standardization for CMS integration (Roberto/Marek collaboration) and serves no real purpose.

## Decision

Remove the index entirely. The new format is `{league}__{year}__{team}`, called `paper_lyt` (League-Year-Team).

## Data Cleanup (already executed)

- **28 `__1` files moved** to `../tdps_rejected/` (markdown + PDF + image dirs), preserving directory structure
- **1 malformed file** (`soccer_simulation_2d__201551__0`) also moved to rejected
- **~3800 `__0` files renamed** to drop the `__0` suffix across both `tdps_markdown/` and `tdps_pdf/`
- **1 outlier** image path fixed inside `rescue_simulation_virtual__2010__IUST.md`
- For the 4 legitimate dual-paper pairs, we keep `__0` and drop `__1` — acceptable loss given corpus size

## Naming Convention

| Before | After |
|--------|-------|
| `lyti` | `paper_lyt` |
| `league_year_team_idx` | `paper_lyt` |
| `league_year_team_indexes` | `paper_lyts` |
| `lyti_filter` | `paper_lyt_filter` |
| `get_filename()` | `get_paper_lyt()` |
| `soccer_smallsize__2024__RoboTeam_Twente__0` | `soccer_smallsize__2024__RoboTeam_Twente` |

## Changes by Layer

### 1. `data_structures`

**`TDPName`** (`file/tdp_name.rs`):
- Remove `pub index: u32` field
- Remove `index: Option<u32>` from constructor
- `get_filename()` → `get_paper_lyt()`, returns 3-part `league__year__team`
- `TryFrom<&str>` expects 3 `__`-separated fields, not 4
- Remove `TDPParseError::Index` variant
- Update all tests

**`Chunk`** (`intermediate/chunk.rs`):
- Rename `league_year_team_idx` → `paper_lyt`
- `to_uuid()` uses the new string — UUIDs change (reindex required)

**`Filter`** (`filter.rs`):
- Rename field `league_year_team_indexes` → `paper_lyts`
- Rename methods `add_league_year_team_index()` → `add_paper_lyt()`, etc.
- Update `#[schemars(description)]`

**`SectionResult`** (`intermediate/navigation.rs`):
- Rename `lyti` field → `paper_lyt`

### 2. `data_access`

**SQLite** (`metadata/sqlite_client.rs`):
- Remove `idx INTEGER NOT NULL` column from `paper` table
- Rename `lyti TEXT PRIMARY KEY` → `paper_lyt TEXT PRIMARY KEY`
- Remove index extraction/insertion in all queries
- No migration — `make rebuild-index` recreates from scratch

**Qdrant** (`vector/qdrant_client.rs`):
- Rename `KEY_LYTI` → `KEY_PAPER_LYT`, value `"lyti"` → `"paper_lyt"`
- Update payload storage, filter logic, and search result extraction
- No migration — full reindex required

### 3. `data_processing`

**`content_chunker.rs`**:
- `league_year_team_idx: tdp.name.get_filename()` → `paper_lyt: tdp.name.get_paper_lyt()`

**`markdown_parser.rs`**:
- Automatically works once TDPName parsing changes
- Update test fixtures to 3-part format

### 4. `api`

**`search.rs`**:
- Rename `lyti_filter` → `paper_lyt_filter`
- Update schema description and example

**All paper handlers** (`get_paper_info.rs`, `get_abstract.rs`, `get_table_of_contents.rs`, `get_section.rs`, `get_table.rs`, `get_image.rs`, `get_paragraph.rs`):
- Rename `lyti` param → `paper_lyt`
- Update `#[schemars(description)]` examples to 3-part format

**`paper_filter.rs`**, **`paper_navigation.rs`**:
- Rename any lyti references

### 5. `mcp`

**`server.rs`**:
- Rename `CompactChunk.lyti` field → `paper_lyt`
- Update all `#[tool(description)]` strings — new examples without `__0`
- Update server instructions (line ~341) — new paper_lyt definition

### 6. `web`

**Routes**:
- `/api/papers/{lyti}/...` → `/api/papers/{paper_lyt}/...`
- Update PDF serving path construction

**`routes/tdps.rs`**:
- Update comments referencing `{lyti}.md` format

### 7. `frontend`

**`types.ts`**:
- Remove `index: number` from `TDPName`
- Rename `league_year_team_idx` → `paper_lyt`
- Rename `league_year_team_indexes` → `paper_lyts`
- Rename `lyti_filter` → `paper_lyt_filter`

**`PaperCard.svelte`**:
- Simplify `paperId` to 3-part construction
- Remove `#if paper.index > 0` badge

**`paper/[id]/+page.ts`**:
- Rename `lyti` variable → `paper_lyt`
- Update comment

**`api.ts`**:
- Rename `lyti_filter` → `paper_lyt_filter`
- Rename `getPaperInfo(lyti)` → `getPaperInfo(paper_lyt)`

**`markdown.ts`**:
- Rename `preprocessMarkdown(raw, lyti)` → `preprocessMarkdown(raw, paper_lyt)`

### 8. `tools`

- All CLI tools use `TDPName` from filenames — automatically work
- `coverage.rs`: `get_filename()` → `get_paper_lyt()`

### 9. Documentation

**Update:**
- `CLAUDE.md` — TDP naming convention, key terms section
- `README.md` — same sections
- Crate-level `CLAUDE.md` files where they reference lyti
- `REMOVE_INDEX.md` — delete (superseded by this spec)

**Leave as-is:**
- `docs/plans/*.md` — historical planning docs, preserve as-is

## Deployment

1. Merge the code changes
2. Data cleanup already done (files renamed on disk)
3. `make rebuild-index` — recreates Qdrant collection + SQLite DB from renamed files
4. Restart services

## Breaking Changes

- MCP tool schemas change — AI clients will see new parameter names and examples
- API parameter names change (`lyti_filter` → `paper_lyt_filter`)
- Frontend URLs change (`/paper/league__year__team__0` → `/paper/league__year__team`)
- All acceptable — no external consumers depend on current format yet
