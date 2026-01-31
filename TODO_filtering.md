# TODO: Simple Metadata & Search Filtering

Plan to store unique combinations of League, Team, and Year in SQLite to provide quick "options" to the LLM, and update the search tool to use a formal `SearchFilter`.

## Proposed Changes

### [Data Access - Vector]
- **Define `SearchFilter`**: In `data_access/src/vector/mod.rs`, define `pub struct SearchFilter { team: Option<String>, league: Option<String>, year: Option<u32> }`.
- **Update `VectorClient`**: Update `search_chunks` to accept `filter: Option<SearchFilter>`.
- **Update Qdrant**: Translate `SearchFilter` into Qdrant `Filter` conditions in `qdrant_client.rs`.

### [Data Access - Metadata]
- **Update Trait**: Add `get_available_filters()` and `register_metadata(league, team, year)` to `MetadataClient`.
- **Update SQLite**: Add a new table `available_metadata (league TEXT, team TEXT, year INTEGER, UNIQUE(league, team, year))`.

### [MCP Server]
- **Update `search` tool**: Use the new `SearchFilter` in `SearchArgs`.
- **New Tool `get_filter_options`**: Add a tool that lets the LLM see what leagues/teams/years exist based on the SQLite metadata.

## Verification
1. Test SQLite registration and retrieval of unique metadata.
2. Test Qdrant search with team/league filters.
