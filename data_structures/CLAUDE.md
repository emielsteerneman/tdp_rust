## Purpose
Pure domain types shared across all crates. No I/O, no side effects.

## Key Types
- `TDPName` — parsed from `league__year__team` strings. `TryFrom<&str>` handles filenames with optional extensions.
- `League` — enum with 16 variants (Soccer 8, Rescue 4, @Home 3, Industrial 1). Has `name()`/`name_pretty()` dual forms.
- `TeamName` — `name` (underscore-separated) and `name_pretty` (space-separated). Constructor normalizes spaces to underscores.
- `Chunk` — the core search unit: text + dense/sparse embeddings + metadata. `to_uuid()` generates deterministic UUIDs.
- `Filter` — optional search filters (teams, leagues, years, content types, paper_lyts).
- `IDF` — newtype over `HashMap<String, (u32, f32)>` (word → index + IDF score). Derefs to inner map.
- `ContentType` — Text, Table, or Image.
- `MarkdownTDP` — aggregates TDPName + FrontMatter + content items + references.
- `EmbedType` — Dense, Sparse, or Hybrid (default).

## Patterns
- **Dual name forms**: League and TeamName both have machine/pretty name accessors.
- **Flexible parsing**: TryFrom implementations accept multiple input formats. League parses from both `soccer_smallsize` and `Soccer SmallSize`.
- **JsonSchema on everything**: types derive `JsonSchema` for MCP tool schema generation.
- **No external I/O**: this crate is a pure data layer.
