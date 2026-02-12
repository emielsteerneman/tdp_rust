# UX Discovery Strategist Memory

## Project: TDP Search (tdp_rust)
- Domain: RoboCup autonomous robot soccer community
- Product: Search engine for ~2000+ Team Description Papers (TDPs)
- Live at: tdpsearch.com
- Primary user: University students (18-25) competing in RoboCup, 2-3 year tenure

## Key Discovery Findings (2026-02-12)
- Full discovery document: see `discovery-session-2026-02-12.md`
- Primary pain: Finding specific technical knowledge across 2000+ papers
- Most common search pattern: Blind search (no idea which team/league has answer)
- Search queries are short keywords, not natural language
- 90/10 split: paper opens vs searches (may include crawlers)
- Cross-league value is core proposition but adoption is Small Size League only
- "Better than nothing" is current competitive position
- MCP/LLM integration is HYPOTHESIS based on user's own behavior
- Team profiles and code search are HYPOTHESIS with no validated demand

## Priority Order (Validated)
1. Robust Technical Search (fuzzy matching, team-name disambiguation)
2. Intelligent Filtering (league, year, team)
3. Cross-League Discovery (default broad search)
4. MCP/LLM integration (validate demand first)
5. Team Profiles (defer)
