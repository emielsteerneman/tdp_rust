## Purpose
CLI binaries for data management, search, and analytics.

## Binaries
- `initialize` — full ingestion pipeline: load markdown → chunk → embed → store in Qdrant + SQLite.
- `search_by_sentence` — CLI search with `--mode` (dense/sparse/hybrid) and `--type` (text/table/image) flags.
- `smoke_test` — end-to-end verification: searches every (league, year) combo across sparse/dense/hybrid against live Qdrant.
- `activity` — query event database: `summary`, `recent`, `agents` subcommands.
- `generate_team_code` — generate HMAC auth code for a team.
- `set_team_metadata` — upsert team metadata: `--team "Name" --key "key" --value "value"`.
- `coverage` — corpus coverage analysis: `parsing` (PDFs vs markdowns), `indexing` (disk vs DB), `heatmap` (league×year grid), `teams` (missing metadata), `all` (default).

## Shared Utilities (lib.rs)
- `validate_team_name()` — fuzzy matches team name against known teams (Jaro-Winkler, threshold 0.7). Exits on failure with suggestions.
- `get_arg()` — simple CLI flag-value parser.

## Pattern
All binaries load config via `AppConfig::load_from_file("config.toml")` and construct clients via `configuration::helpers`.
