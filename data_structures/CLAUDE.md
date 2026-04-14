## Purpose
Pure domain types shared across all crates. No I/O, no side effects.

## Key Types
- `TDPName` — parsed from `league__year__team` strings. `TryFrom<&str>` handles filenames with optional extensions.
- `League` — enum with 16 variants (Soccer 8, Rescue 4, @Home 3, Industrial 1). Dual forms: `name()`/`name_pretty()`, plus `major()`/`minor()`/`sub()` hierarchy and `all()` static method.
- `TeamName` — `name` (underscore-separated) and `name_pretty` (space-separated). Constructor normalizes spaces to underscores.
- `Chunk` — the core search unit: text + dense/sparse embeddings + metadata (paper_lyt, league, year, team, content_seq, chunk_seq, content_type, title, image_path). `to_uuid()` generates deterministic UUIDs.
- `ChunkMetadata` — metadata-only view of a Chunk (no embeddings).
- `Filter` — optional search filters using `HashSet` fields (teams, leagues, years, content types, paper_lyts). Builder methods: `add_team()`, `add_league()`, etc.
- `IDF` — newtype over `HashMap<String, (u32, f32)>` (word → index + IDF score). Derefs to inner map.
- `ContentType` — Text, Table, or Image.
- `ContentItem` — individual content unit with seq, type, depth, title, body, image_path.
- `FrontMatter` — title, authors, institutions, URLs, abstract_text.
- `Author` — name + affiliation.
- `PaperInfo` — summary metadata (title, authors, institutions, URLs).
- `MarkdownTDP` — aggregates TDPName + FrontMatter + content items + references + raw_markdown.
- `TocEntry` — table of contents entry.
- `EmbedType` — Dense, Sparse, or Hybrid (default).

## Intermediate Types (intermediate module)
- `SearchResult` — complete search result with query, filter, chunks, suggestions, highlight_terms.
- `SearchResultChunk` — individual chunk with score and breadcrumbs.
- `BreadcrumbEntry` / `SectionResult` — section hierarchy navigation.

## Patterns
- **Dual name forms**: League and TeamName both have machine/pretty name accessors.
- **Flexible parsing**: TryFrom implementations accept multiple input formats. League parses from both `soccer_smallsize` and `Soccer SmallSize`.
- **JsonSchema on everything**: types derive `JsonSchema` for MCP tool schema generation.
- **No external I/O**: this crate is a pure data layer.
