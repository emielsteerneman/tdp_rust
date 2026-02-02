# TDP Search System - Persistent Project State

This document serves as a "long-term memory" for tool development, evaluation, and test history. Use this to resume context on the search system's capabilities and goals.

## 🛠 Desired Search Tools (TODO List)

| Tool | Status | Description |
| :--- | :--- | :--- |
| `get_available_teams` | 🚧 Planned | Allows fuzzy searching for team names with optional league filtering. Prevents "blind" filtering errors. |
| `list_leagues` | ⏳ Todo | Returns a list of all available leagues in the corpus to help build higher-level filters. |
| `get_full_tdp` | ⏳ Todo | Fetches the entire text of a TDP by its `league_year_team_idx`. Crucial for reading context around discovered chunks. |
| `search_specs` | ⏳ Todo | A specialized search optimized for tables and structured technical specifications. |
| `compare_teams` | ⏳ Todo | Aggregates results for multiple specified teams/years into a single view. |

---

## 🧪 Benchmark Test: "Battery Capacity" (2026-02-02)

### Objective
Evaluate the hybrid search's ability to extract specific technical parameters (battery capacity) across different teams and years.

### Methodology
1.  **Global Search:** `query: "battery capacity", search_type: "HYBRID"`
2.  **Refined Search (TIGERs):** Using suggestion `TIGERs Mannheim`.
3.  **Refined Search (RoboTeam):** Using suggestions `RoboTeam Twente`.

### Results
*   **URoboRus (2019):** 3000 mAh, 26V average (found via global search).
*   **MRL (2013):** 1750 mAh, 5-cells LiPo (found via global search).
*   **TIGERs Mannheim (2020):** 1300 mAh, 6S series (found via targeted filter).
*   **RoboTeam Twente (2023/2024):** 6S1P 22.2V 150C LiPo. *Note: Exact mAh capacity was not in the technical tables for these years.*

### Rerun Instructions
Run the following tool calls to verify results:
```json
// Global search
mcp_tdp_search_search(query: "battery capacity", search_type: "HYBRID")

// Targeted TIGERs
mcp_tdp_search_search(query: "battery capacity", filter: { teams: ["TIGERs Mannheim"] }, search_type: "HYBRID")

// Targeted RoboTeam
mcp_tdp_search_search(query: "Technical Specifications Table battery capacity", filter: { teams: ["RoboTeam Twente"] }, search_type: "HYBRID")
```

---

## 🧠 Missing Capabilities Checklist
- [ ] **Metadata Discovery:** Navigating team/league identifiers is currently a trial-and-error process. 
- [ ] **Context Expansion:** Chunks are too small to capture full spec tables that span multiple paragraphs.
- [ ] **Fuzzy Filtering:** Filters currently require case-sensitive exact matches.
