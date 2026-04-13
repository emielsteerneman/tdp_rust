# Connect your scripts — `/connect/api` page

## Summary

A new frontend page at `/connect/api` that helps developers and researchers connect to the TDP Search REST API via Python scripts, curl, or any HTTP client. Mirrors the style and structure of the existing `/connect` (MCP) page. No authentication required.

## Route

`/connect/api/+page.svelte` — alongside the existing `/connect/+page.svelte`.

## Page sections

### 1. Header

- Title: "Connect your scripts"
- Subtitle: "The TDP Search REST API gives you programmatic access to 2000+ RoboCup papers. No authentication required."

### 2. Base URL

Copyable code block:

```
https://web.emielsteerneman.nl/api
```

Note beneath: "All endpoints return JSON."

### 3. Quick examples

Three cards, each showing both a Python (`requests`) snippet and a curl snippet:

- **Search papers** — `GET /api/search?q=ball+detection&league=soccer_smallsize`
- **Read a paper abstract** — `GET /api/papers/soccer_smallsize__2024__RoboTeam_Twente/abstract`
- **List teams** — `GET /api/teams`

Python examples use the `requests` library. Curl examples use `curl -s ... | python -m json.tool` for pretty-printing.

### 4. Filter values reference

Live-fetched from layout data (already preloaded — zero extra API calls):

- **Leagues** — all valid league machine names (e.g. `soccer_smallsize`, `soccer_middlesize`, `rescue_robot`, ...)
- **Years** — all valid years
- **paper_lyt format** — explain the `{league}__{year}__{team}` double-underscore convention with a concrete example

### 5. All endpoints

Rendered list of all API endpoints: method, path, one-line description. Content is hardcoded (curated editorial), not fetched from `/api`. Links to the raw JSON at `/api` for programmatic consumption.

## Navbar change

- Update the "Connect your AI" button text to **"Connect your AI / scripts"**
- Link stays `/connect`
- Both pages are discoverable from each other via a tab/link pair at the top of each page ("Connect your AI" | "Connect your scripts")

## Styling

Matches the existing `/connect` page exactly: same card borders, spacing, dark mode treatment, Tailwind classes.

## Data sources

- **Filter values (leagues, years):** live from layout data (already available via `+layout.ts` preload)
- **Endpoint list and examples:** hardcoded in the component (editorial content)

## What this does NOT include

- No interactive "try it" widget
- No JavaScript examples (curl covers that audience)
- No new API endpoints
- No changes to the `/api` JSON response
