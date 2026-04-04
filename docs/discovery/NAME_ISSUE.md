# Issue: Inconsistent name / name_pretty usage across storage layers

## Summary

The League enum refactor (commit f9f5e24) changed league storage from pretty names to machine names in Qdrant and SQLite, but did not apply the same change to team names. Additionally, the CLAUDE.md convention was not updated to reflect the new approach. Test fixtures still insert pretty names, creating a mismatch with production code paths.

## Current state

| Location | League stored as | Team stored as |
|----------|-----------------|----------------|
| Qdrant payload (write) | `name()` — machine (`soccer_smallsize`) | `name_pretty` — pretty (`RoboTeam Twente`) |
| Qdrant filter (read) | `name()` — machine | n/a (teams filtered differently) |
| SQLite `store_paper` | `name()` — machine | `name_pretty` — pretty |
| SQLite test fixtures | pretty (`"Soccer SmallSize"`) | pretty (`"RoboTeam Twente"`) |
| SQLite `load_leagues` | parses via `League::try_from` (accepts both) | returns `TeamName` struct |
| MCP/Web boundary | `name_pretty()` for display | `name_pretty` for display |

## Problems

### 1. League and team are stored differently in the same payload block

In `data_access/src/vector/qdrant_client.rs:223-227`:
```rust
// League Year Team
payload.insert(Self::KEY_LEAGUE.into(), chunk.league.name().into());      // machine
payload.insert(Self::KEY_YEAR.into(), (chunk.year as i64).into());
payload.insert(Self::KEY_TEAM.into(), chunk.team.name_pretty.into());     // pretty
```

League uses `name()` (machine) while team uses `name_pretty` (pretty) in the same block. Same pattern in `sqlite_client.rs:393-395`.

### 2. SQLite test fixtures use pretty names, production code writes machine names

`sqlite_client.rs` test at line 787 inserts `"Soccer SmallSize"` (pretty) directly via SQL, but `store_paper` at line 393 writes `league.name()` (machine). Both parse correctly via `League::try_from`, so tests pass — but the stored values differ between test and production paths.

### 3. CLAUDE.md convention is stale

CLAUDE.md line 27 says:
> Qdrant payloads use pretty names; file keys use machine names.

This no longer reflects reality for the league field.

## Proposed fix

Pick one convention and apply it consistently:

**Option A: Machine names everywhere internally (recommended)**
- Update `TeamName` storage in Qdrant and SQLite to also use machine names
- Update CLAUDE.md to say "Internal storage uses machine names; pretty names are used only at the MCP/web boundary for display"
- Update SQLite test fixtures to use machine names
- Ensure `TeamName` has a `name()` method returning the machine name (currently only has `name_pretty`)

**Option B: Pretty names everywhere internally**
- Revert league storage in Qdrant and SQLite to `name_pretty()`
- Update Qdrant filter to use `name_pretty()` for league matching
- Keep CLAUDE.md as-is

Option A aligns with the design intent described during the League refactor (commit f9f5e24) and keeps the pretty-name boundary at the web/mcp inlet layer.

## Files to change

For Option A:
- `data_access/src/vector/qdrant_client.rs` — change team storage to machine name
- `data_access/src/metadata/sqlite_client.rs` — change team storage to machine name, update test fixtures
- `data_structures/src/file/team_name.rs` — add `name()` method if not present
- `CLAUDE.md` — update the "Dual name forms" convention
