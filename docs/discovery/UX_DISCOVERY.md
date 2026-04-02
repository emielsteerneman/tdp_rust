# UX Discovery Session - TDP Search - 2026-02-12

## Context
TDP Search is a search engine for RoboCup Team Description Papers (~2000+ papers).
The builder (emiel) is both the creator and a primary user (Small Size League competitor).
Live at: tdpsearch.com

## Primary Persona: "The Rotating Researcher"
- University student, 18-25, bachelor's/master's
- Active RoboCup competitor, 2-3 year tenure
- High technical literacy within their domain, low cross-league awareness
- Hits technical problems while building robots, needs to find how others solved them
- Searches with short keywords (1-3 terms), not natural language
- Most common: blind search across entire corpus (doesn't know which team/league)
- Less common but important: targeted search within known team's papers

## Validated Findings
- 2000+ papers is unmanageable without search tooling
- Search quality issues: hyphenation mismatch ("bang bang" vs "bang-bang")
- Team names as search noise (citing team vs papers BY team)
- Users default to filtering by own league even in cross-league tool
- Users iterate on failed queries (reformulate, add terms) rather than abandoning
- High community turnover means papers are critical persistent knowledge store
- Current tool valued as "better than having nothing" -- low bar, high opportunity

## Hypotheses (Not Validated)
- **[PARTIALLY VALIDATED]** Other leagues feel same pain (personal experience + some feedback)
- **[HYPOTHESIS]** MCP/LLM integration matches user workflow (based on user's own behavior only)
- **[HYPOTHESIS]** Discord bot has utility beyond promotion
- **[HYPOTHESIS]** Team profiles with GitHub links are wanted
- **[HYPOTHESIS]** Searching through team source code is valuable

## Concrete Pain Point Example
User searched for "bang bang tigers" to find trajectory planning info:
1. "bang bang" didn't match "bang-bang" (hyphenation)
2. "tigers" returned papers citing Tigers, not papers BY Tigers
3. Workaround: manually added Tigers to team filter, searched "bang bang bang-bang"
4. Required insider knowledge of search behavior -- typical user wouldn't know this

## Key Behavioral Insights
- 90/10 paper opens vs searches (may include crawlers)
- Most searches filtered to Small Size League
- Common queries: keywords, team names, "topic + team name" combos
- Users browse papers (by year/league) when they don't have specific query
- No fallback to Discord or Google -- users stay in tool and reformulate

## Success Criteria
- Conversational MCP search patterns in logs
- Focused searches with proper filters
- Multi-league adoption (not just Small Size League)

## Three Key Takeaways

1. **Search quality IS the product.** The "bang-bang" example isn't an edge case -- it's the core experience. When the most common use case is blind search across the whole corpus, every search quality failure is a dead end with no fallback.

2. **Cross-league discovery is the unique moat, but behavior works against it.** Users default to their own league filter because that's what feels safe. Surfacing an answer from a league the user has never heard of requires designing against the user's default instinct -- a behavioral design challenge, not a feature checkbox.

3. **Resist scope expansion until the core is excellent.** MCP, Discord bots, team profiles, and code search are all interesting ideas, but none have validated demand from the community. The path to strong adoption and organic word-of-mouth is making the core search experience so good that people tell their friends about it. Everything else comes after that.

## Recommended Priority Order
1. Robust Technical Search (fuzzy matching, disambiguation)
2. Intelligent Filtering (league, year, team)
3. Cross-League Discovery (default broad search)
4. MCP/LLM Knowledge Base (validate demand first)
5. Team Profiles (defer)
