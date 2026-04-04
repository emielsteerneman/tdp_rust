# Deep Dive: GitHub Repo Browsing Through Our MCP Server

## Problem

Claude on claude.ai is bad at browsing GitHub repos. We already have an MCP server that Claude connects to for TDP search. Can we expose GitHub repo browsing capabilities through that same server, so it feels like one integrated experience?

## Research Summary

We investigated: the official GitHub MCP server, MCP proxy/gateway projects, the rmcp framework's capabilities, and creative alternatives.

---

## The GitHub MCP Server (`github/github-mcp-server`)

- **Docker image**: `ghcr.io/github/github-mcp-server` (public, on GHCR)
- **Transports**: stdio (default) and Streamable HTTP (`github-mcp-server http`, port 8082)
- **Remote hosted**: GitHub runs one at `https://api.githubcopilot.com/mcp/` (requires GitHub Copilot OAuth)
- **Auth**: GitHub Personal Access Token (PAT) via `GITHUB_PERSONAL_ACCESS_TOKEN` env var, or OAuth
- **Tools**: ~40 tools across 19 toolsets (repos, issues, PRs, actions, code security, etc.)
- **Configurable**: `--toolsets repos,issues` or `--tools get_file_contents,search_code` to limit exposed tools
- **No SSE**: Only stdio and Streamable HTTP. No legacy SSE transport.
- **Gotchas**: Docker image defaults to `stdio` CMD; need explicit `http` arg for HTTP mode. `--read-only` overrides `--tools`. Different toolsets need different PAT scopes.

## Our MCP Server Architecture

- **Framework**: rmcp 0.17.0 (Rust)
- **Transport**: Streamable HTTP via Axum on :50001 (open) and :50002 (OAuth)
- **Tool registration**: Compile-time via `#[tool_router]` and `#[tool]` procedural macros
- **Tools are static**: Cannot add tools at runtime without code changes (but `ServerHandler` can be implemented manually for dynamic dispatch)
- **State sharing**: All expensive clients (embeddings, vectors, metadata, search) are `Arc`'d and shared across sessions
- **Pattern**: Each `#[tool]` method calls an `api` handler. All business logic lives in `api`.

## MCP Proxy/Gateway Ecosystem

The ecosystem for composing multiple MCP servers has matured:

| Project | Language | What it does |
|---------|----------|-------------|
| **tbxark/mcp-proxy** | Go | Aggregates multiple servers behind one HTTP endpoint. Supports stdio + SSE + Streamable HTTP |
| **metatool-ai/metamcp** | TypeScript | Namespaced tool aggregation with middleware/observability |
| **DXHeroes/local-mcp-gateway** | TypeScript/NestJS | Profile-based tool selection, SQLite, nice UI |
| **zach-source/mcp-rust-proxy** | Rust | Connection pooling, health monitoring |
| **adamwattis/mcp-proxy-server** | - | Aggregates multiple MCP resource servers via JSON config |
| **VeriTeknik/pluggedin-mcp-proxy** | - | "Manages all your other MCPs in one MCP" |
| **igrigorik/MCProxy** | - | Tool aggregation, search, filtering, and security |
| **Kong AI Gateway 3.14** | - | Enterprise API gateway with `ai-mcp-proxy` plugin |

The MCP spec does not define a formal "proxy" primitive, but explicitly mentions server composition as a planned feature. Nothing stops a process from being both a server (downstream) and a client (upstream).

rmcp supports this: enable the `"client"` feature to get `RoleClient` / `Peer<RoleClient>`, call `list_tools()` on upstream servers, and implement `ServerHandler` manually (bypassing the macro-based router) for dynamic tool dispatch.

---

## All Options, Ranked

### Option 1: Add GitHub API Tools Directly to Our Server (Best ROI)

Add 4-5 tools that call GitHub's REST API directly. No proxy, no Docker, no extra infrastructure.

**Clever trick**: Use the REST API for `get_repository_tree` (1 API call, returns entire file tree), then `raw.githubusercontent.com` for file reads (CDN, no auth needed for public repos, no rate limits). Burns almost zero API quota.

| Tool | Backend | Rate limit impact |
|------|---------|-------------------|
| `github_repo_tree` | REST API `GET /repos/{o}/{r}/git/trees/{sha}?recursive=1` | 1 call per repo |
| `github_read_file` | `raw.githubusercontent.com/{o}/{r}/{branch}/{path}` | **Zero** (CDN) |
| `github_search_code` | REST API `GET /search/code` | 30/min |
| `github_get_readme` | REST API `GET /repos/{o}/{r}/readme` | Negligible |

**Killer integration**: Claude calls `get_team_info("TIGERs Mannheim")` -> gets their GitHub URL -> calls `github_repo_tree` -> browses their code -> cross-references with TDP search results. Papers + code in one conversation.

**Implementation**: ~200 lines of Rust, follows existing `api` handler pattern. Consider `octocrab` (mature async Rust GitHub client) or raw `reqwest`.

**Effort**: Half a day.

### Option 2: Use an Existing MCP Proxy

Run something like `mcp-proxy` (Go) in Docker alongside our server. Configure it with two upstreams: our `:50001` and GitHub MCP server. Expose the proxy as the single endpoint Claude connects to.

**Pros**: Zero code changes. Access to all 40 GitHub tools.

**Cons**: Adds a moving piece. Dependency on third-party proxy. Operational complexity.

**When to choose**: If you want *all* GitHub tools without writing code.

### Option 3: Docker GitHub MCP Server + Built-in Proxy

Run `ghcr.io/github/github-mcp-server` in Docker. Enable rmcp's `"client"` feature in our server. Connect to the GitHub server as a subprocess. Call `list_tools()` to discover tools, route `call_tool()` dynamically by implementing `ServerHandler` manually.

**Pros**: Everything in one Rust binary. Full GitHub tool access.

**Cons**: Way more work than Option 1 for the same result. Must maintain MCP client code.

### Option 4: Clone Repos + Local Grep (The Power Move)

Shallow-clone team repos to disk. Serve files directly. Run ripgrep for code search.

**Tools**: `clone_repo(owner, repo)`, `read_repo_file(owner, repo, path)`, `grep_repo(owner, repo, pattern, path_glob)`

**Why this is interesting**: Grep is actually better than GitHub's code search for many queries. A shallow clone of most RoboCup team repos is <100MB each.

**Variant**: Lazy clone on first request, cache with TTL.

**Effort**: 1-2 days.

### Option 5: Index Repos Into Qdrant (The Moonshot)

Our pipeline already does `markdown -> chunk -> embed -> Qdrant`. Build a variant: `source code -> chunk -> embed -> Qdrant "code" collection`. Then search finds both papers AND code.

Imagine: "find teams that describe trajectory planning in their TDP and implement it in their GitHub repo."

**Pros**: Genuinely unique. Semantic code search. No other MCP server does this.

**Cons**: Larger project. Requires cloning and re-indexing when repos change. Storage overhead.

### Option 6: Connect GitHub's Remote Server Directly from Claude.ai

Claude.ai supports multiple MCP servers. Just add `https://api.githubcopilot.com/mcp/` as a second connection.

**Caveat**: Requires GitHub Copilot OAuth, which may not work from claude.ai. Worth trying for 5 minutes before building anything.

---

## Recommendation

### Phase 1 (this week): Option 1

Add 4 GitHub tools to our server. Use `raw.githubusercontent.com` for file reads to avoid rate limits. Wire it into `get_team_info` so Claude can go from "team X" -> their code in one flow. Half a day of work.

### Phase 2 (if we want more): Option 4

Add `clone_repo` + `grep_repo` for repos we frequently browse. Gives us grep superpowers that GitHub's API can't match.

### Phase 3 (moonshot): Option 5

Semantic code search in Qdrant. Papers + source code searchable from one query. This would be a differentiating feature for the whole project.

### Skip

The proxy approaches (Options 2 and 3). They add complexity for something we can build natively in our existing architecture with less effort.
