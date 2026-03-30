# PDF Serving Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve original PDF files for indexed TDPs and add a "View Original PDF" button to the paper detail page.

**Architecture:** Mirror the existing `/tdps/` static file serving pattern with a new `/pdfs/` route backed by `tdps_pdf_root`. Add a `PdfOpen` event for analytics. Frontend swaps the disabled placeholder button for a live link.

**Tech Stack:** Rust/Axum (backend), Svelte 5 (frontend), event_processing crate (analytics)

---

### Task 1: Add `tdps_pdf_root` to config

**Files:**
- Modify: `data_processing/src/config.rs:4-6`
- Modify: `configuration/src/appconfig.rs:87-88` (test TOML strings)
- Modify: `configuration/src/appconfig.rs:125-126` (test TOML strings)

- [ ] **Step 1: Add field to `DataProcessingConfig`**

In `data_processing/src/config.rs`, add `tdps_pdf_root`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
}
```

- [ ] **Step 2: Update config test TOML strings**

In `configuration/src/appconfig.rs`, add `tdps_pdf_root = "some_pdf_root"` after every `tdps_markdown_root = "some_root"` line in both test functions (`test_simple_config` and `test_full_config`).

In `test_simple_config`, add an assertion:

```rust
assert_eq!(config.data_processing.tdps_pdf_root, "some_pdf_root");
```

- [ ] **Step 3: Run tests to verify**

Run: `cargo test -p configuration`
Expected: PASS — both config tests parse the new field.

- [ ] **Step 4: Commit**

```bash
git add data_processing/src/config.rs configuration/src/appconfig.rs
git commit -m "feat: add tdps_pdf_root to DataProcessingConfig"
```

---

### Task 2: Add `tdps_pdf_root` to `AppState` and wire in `main.rs`

**Files:**
- Modify: `web/src/state.rs`
- Modify: `web/src/main.rs:71-77`

- [ ] **Step 1: Add field to `AppState`**

In `web/src/state.rs`, add `tdps_pdf_root: String` to the struct and constructor:

```rust
#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        tdps_markdown_root: String,
        tdps_pdf_root: String,
        team_registry: Option<Arc<dyn TeamRegistryClient + Send + Sync>>,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            tdps_markdown_root,
            tdps_pdf_root,
            team_registry,
        }
    }
}
```

- [ ] **Step 2: Pass `tdps_pdf_root` in `main.rs`**

In `web/src/main.rs`, update the `AppState::new()` call (around line 71):

```rust
    let state = AppState::new(
        metadata_client.clone(),
        Arc::new(searcher),
        dispatcher,
        config.data_processing.tdps_markdown_root.clone(),
        config.data_processing.tdps_pdf_root.clone(),
        team_registry,
    );
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p web`
Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/state.rs web/src/main.rs
git commit -m "feat: wire tdps_pdf_root through AppState"
```

---

### Task 3: Add `PdfOpenEvent` to event_processing

**Files:**
- Modify: `event_processing/src/lib.rs`
- Modify: `event_processing/src/listeners/telegram.rs`

- [ ] **Step 1: Add `PdfOpenEvent` struct and enum variant**

In `event_processing/src/lib.rs`, add the struct after `PaperOpenEvent`:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct PdfOpenEvent {
    pub paper_id: String,
}
```

Add the variant to `Event` enum (after `PaperOpen`):

```rust
    PdfOpen(PdfOpenEvent),
```

Add the match arm in `event_type()` (after `PaperOpen`):

```rust
            Event::PdfOpen(_) => "pdf_open",
```

- [ ] **Step 2: Add test case to `test_event_type_strings`**

Add to the `cases` vec in the existing test:

```rust
            (Event::PdfOpen(PdfOpenEvent { paper_id: "p".into() }), "pdf_open"),
```

- [ ] **Step 3: Add serialization test case**

Add to the `events` vec in `test_event_serialization_all_variants`:

```rust
            Event::PdfOpen(PdfOpenEvent { paper_id: "test_paper".into() }),
```

- [ ] **Step 4: Add Telegram formatting**

In `event_processing/src/listeners/telegram.rs`, add a match arm in `format_message()` after the `PaperOpen` arm:

```rust
            Event::PdfOpen(e) => {
                Some(format!("[{src}] PDF opened: {}", e.paper_id))
            }
```

- [ ] **Step 5: Add Telegram format test**

Add a new test in `event_processing/src/listeners/telegram.rs`:

```rust
    #[test]
    fn format_pdf_open() {
        let listener = make_listener();
        let event = Event::PdfOpen(PdfOpenEvent {
            paper_id: "soccer_smallsize__2024__RoboTeam__0".into(),
        });

        let msg = listener
            .format_message(&EventSource::Web, &event)
            .unwrap();
        assert!(msg.contains("[web]"));
        assert!(msg.contains("PDF opened"));
        assert!(msg.contains("soccer_smallsize__2024__RoboTeam__0"));
    }
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p event_processing`
Expected: PASS — all existing tests plus the new ones.

- [ ] **Step 7: Commit**

```bash
git add event_processing/src/lib.rs event_processing/src/listeners/telegram.rs
git commit -m "feat: add PdfOpenEvent to event system"
```

---

### Task 4: Add `serve_pdf_file` route handler

**Files:**
- Create: `web/src/routes/pdfs.rs`

- [ ] **Step 1: Create the handler**

Create `web/src/routes/pdfs.rs`:

```rust
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use data_structures::file::TDPName;

use crate::state::AppState;

pub async fn serve_pdf_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // Strip .pdf extension
    let lyti_str = path
        .strip_suffix(".pdf")
        .ok_or(StatusCode::BAD_REQUEST)?;

    let tdp_name = TDPName::try_from(lyti_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build filesystem path
    let root = std::path::Path::new(&state.tdps_pdf_root);
    let league = &tdp_name.league;
    let year = tdp_name.year.to_string();

    let league_path = if let Some(sub) = league.sub() {
        root.join(league.major())
            .join(league.minor())
            .join(sub)
            .join(&year)
    } else {
        root.join(league.major())
            .join(league.minor())
            .join(&year)
    };

    let file_path = league_path.join(format!("{}.pdf", lyti_str));

    // Security: canonicalize and verify under root
    let canonical_root = std::fs::canonicalize(root).map_err(|_| StatusCode::NOT_FOUND)?;
    let canonical_file = std::fs::canonicalize(&file_path).map_err(|_| StatusCode::NOT_FOUND)?;

    if !canonical_file.starts_with(&canonical_root) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Read and return PDF
    let contents = tokio::fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/pdf"),
        )
        .body(Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p web`
Expected: warning about unused module (not registered yet), but no errors.

- [ ] **Step 3: Commit**

```bash
git add web/src/routes/pdfs.rs
git commit -m "feat: add serve_pdf_file route handler"
```

---

### Task 5: Add `pdf_open_handler` and register all new routes

**Files:**
- Modify: `web/src/routes/papers.rs`
- Modify: `web/src/routes/mod.rs`

- [ ] **Step 1: Add `pdf_open_handler` to `papers.rs`**

Add at the end of `web/src/routes/papers.rs`:

```rust
pub async fn pdf_open_handler(
    State(state): State<AppState>,
    Path(lyti): Path<String>,
) -> StatusCode {
    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::PdfOpen(event_processing::PdfOpenEvent {
            paper_id: lyti,
        }),
    );

    StatusCode::NO_CONTENT
}
```

- [ ] **Step 2: Register routes in `mod.rs`**

In `web/src/routes/mod.rs`, add the module declaration at the top:

```rust
mod pdfs;
```

Add the PDF open route to `api_routes` (after the existing `papers/{id}/open` route):

```rust
        .route("/api/papers/{id}/pdf-open", post(papers::pdf_open_handler))
```

Add the pdfs route group (after `tdps_routes`):

```rust
    let pdfs_routes = Router::new()
        .route("/pdfs/{*path}", get(pdfs::serve_pdf_file))
        .with_state(state.clone());
```

Merge it in the final Router (after `.merge(tdps_routes)`):

```rust
        .merge(pdfs_routes)
```

Note: the `tdps_routes` `.with_state(state)` needs to change to `.with_state(state.clone())` since `state` is now also used by `pdfs_routes`.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p web`
Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/routes/papers.rs web/src/routes/mod.rs
git commit -m "feat: register /pdfs/ route and pdf-open endpoint"
```

---

### Task 6: Frontend — replace disabled button with live PDF link

**Files:**
- Modify: `frontend/src/routes/paper/[id]/+page.svelte`

- [ ] **Step 1: Move the PDF link to the top and make it functional**

In `frontend/src/routes/paper/[id]/+page.svelte`, replace the entire `<!-- Actions -->` block (the `<div class="flex justify-center">` with the disabled button inside it, lines 63-77) with nothing — remove it.

Then, inside the paper content area, add a PDF link at the top. Replace the opening of the article's parent div:

```svelte
			<div class="bg-white dark:bg-gray-900 rounded-lg shadow-sm border border-gray-200 dark:border-gray-800 p-4 sm:p-6 md:p-8 mb-4 sm:mb-6">
```

with:

```svelte
			<div class="bg-white dark:bg-gray-900 rounded-lg shadow-sm border border-gray-200 dark:border-gray-800 p-4 sm:p-6 md:p-8 mb-4 sm:mb-6">
				<div class="flex justify-end mb-4">
					<a
						href="/pdfs/{data.lyti}.pdf"
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
						onclick={() => {
							fetch(`/api/papers/${encodeURIComponent(data.lyti)}/pdf-open`, { method: 'POST' }).catch(() => {});
						}}
					>
						View Original PDF
					</a>
				</div>
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd frontend && npm run build`
Expected: builds without errors.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/routes/paper/[id]/+page.svelte
git commit -m "feat: add View Original PDF button to paper page"
```

---

### Task 7: Add `/pdfs` proxy to Vite dev config

**Files:**
- Modify: `frontend/vite.config.ts`

- [ ] **Step 1: Add the proxy entry**

In `frontend/vite.config.ts`, add a `/pdfs` proxy entry after the `/tdps` entry:

```typescript
			'/pdfs': {
				target: 'http://localhost:50000',
				changeOrigin: true
			}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/vite.config.ts
git commit -m "feat: add /pdfs proxy to vite dev config"
```

---

### Task 8: Update config files and documentation

**Files:**
- Modify: `config.toml.example`
- Modify: `config.docker.toml`
- Modify: `docker-compose.yml`
- Modify: `.env.example`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update `config.toml.example`**

Add `tdps_pdf_root` after `tdps_markdown_root`:

```toml
[data_processing]
tdps_markdown_root = "/path/to/tdps_markdown/"
tdps_pdf_root = "/path/to/tdps_pdf/"
```

- [ ] **Step 2: Update `config.docker.toml`**

Add `tdps_pdf_root` after `tdps_markdown_root`:

```toml
[data_processing]
tdps_markdown_root = "/data/tdps_markdown"
tdps_pdf_root = "/data/tdps_pdf"
```

- [ ] **Step 3: Update `docker-compose.yml`**

Add a PDF volume mount to the `web` service, after the markdown mount:

```yaml
      - ${TDP_PDF_ROOT:?Set TDP_PDF_ROOT in .env}:/data/tdps_pdf:ro
```

- [ ] **Step 4: Update `.env.example`**

Add after `TDP_MARKDOWN_ROOT`:

```
# Path to TDP PDF files on host (required, mounted read-only into web container)
TDP_PDF_ROOT=/path/to/tdps_pdf
```

- [ ] **Step 5: Update `CLAUDE.md`**

In the config example under "Local Setup Prerequisites", add `tdps_pdf_root` after `tdps_markdown_root`:

```toml
[data_processing]
tdps_markdown_root = "/path/to/tdps_markdown/"
tdps_pdf_root = "/path/to/tdps_pdf/"
```

Also add to the prerequisites list:

```
- **TDP PDF files** must exist at `tdps_pdf_root` for the "View Original PDF" button to work
```

- [ ] **Step 6: Commit**

```bash
git add config.toml.example config.docker.toml docker-compose.yml .env.example CLAUDE.md
git commit -m "docs: add tdps_pdf_root to config examples and documentation"
```

---

### Task 9: Update local `config.toml` and verify end-to-end

**Files:**
- Modify: `config.toml` (gitignored, local only)

- [ ] **Step 1: Add `tdps_pdf_root` to local config**

Add to `config.toml` under `[data_processing]`:

```toml
tdps_pdf_root = "/home/emiel/projects/tdps_pdf"
```

- [ ] **Step 2: Run all backend tests**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 3: Manual smoke test**

Start the web server (`cargo run -p web`) and verify:

1. Open a paper page in the browser — PDF button visible at top
2. Click "View Original PDF" — PDF opens in new tab
3. Check server logs — `PdfOpen` event dispatched
4. Try an invalid path like `/pdfs/nonexistent.pdf` — returns 404
5. Try a path traversal like `/pdfs/../../etc/passwd.pdf` — returns 400 or 404

- [ ] **Step 4: Verify frontend dev proxy**

Run `cd frontend && npm run dev`, visit a paper page, and confirm the PDF button works through the Vite proxy.
