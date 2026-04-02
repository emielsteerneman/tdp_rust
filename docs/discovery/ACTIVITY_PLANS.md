# Activity Logging Cleanup

## Problem
A single user search generates ~18 events (6 HTTP requests × 2 events each: http_request + domain event).
Most of these are noise — sidebar data fetches and low-level HTTP logging.

## What to keep
Meaningful user-action events only:
- `search` — user searched for something
- `get_abstract`, `get_section`, `get_table_of_contents`, `get_tdp_contents` — user opened/read a paper
- `get_image`, `get_table`, `get_paragraph` — user viewed specific content

## What to remove
- `http_request` middleware logging (`web/src/middleware.rs`) — low-level, duplicates domain events
- `list_leagues`, `list_teams`, `list_years`, `list_papers` — page hydration, not user intent

## Future: richer context
- Track referrer/source page (did user come from frontpage browse or search results?)
- IP and user_agent are currently only in `http_request` events — if needed later, pass request context into api handlers
