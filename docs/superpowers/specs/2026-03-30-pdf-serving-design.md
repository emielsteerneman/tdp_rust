# PDF Serving — Design Spec

Serve original PDF files for indexed TDPs via the web server, with a "View Original PDF" button on the paper detail page.

## Context

TDP markdown files are parsed from original PDFs. The markdown is indexed and searchable; the PDF is the canonical source document. Both live on disk in mirrored directory structures:

- Markdown: `{tdps_markdown_root}/{major}/{minor}/{year}/{lyti}.md`
- PDF: `{tdps_pdf_root}/{major}/{minor}/{year}/{lyti}.pdf`

Every indexed markdown has a corresponding PDF. Not all PDFs have been parsed to markdown yet, but that's fine — only indexed papers appear on the web, and those always have a PDF.

## Scope

Web only. No MCP changes. No new business-logic endpoints in the `api` crate — PDF serving is static file serving, and the PDF open tracking is a fire-and-forget event dispatch (same pattern as `paper_open`).

## Design

### Backend

**Config** — Add `tdps_pdf_root: String` to `DataProcessingConfig` (required field, not optional):

```toml
[data_processing]
tdps_markdown_root = "/path/to/tdps_markdown/"
tdps_pdf_root = "/path/to/tdps_pdf/"
```

**State** — Add `tdps_pdf_root: String` to `AppState`. Populated from `config.data_processing.tdps_pdf_root` in `main.rs`.

**Route** — New file `web/src/routes/pdfs.rs` with handler `serve_pdf_file`:

- Mounted at `/pdfs/{*path}` in `routes/mod.rs` (alongside `/tdps/{*path}`, no activity logging)
- Accepts paths like `{lyti}.pdf`
- Strips `.pdf` extension, parses `TDPName`, builds filesystem path using league hierarchy
- Canonicalize path + verify it's under `tdps_pdf_root` (path traversal protection)
- Returns file with `Content-Type: application/pdf`
- No subdirectory/image logic needed (unlike the markdown handler)

**Route registration** in `routes/mod.rs`:

```rust
let pdfs_routes = Router::new()
    .route("/pdfs/{*path}", get(pdfs::serve_pdf_file))
    .with_state(state.clone());
```

Merged alongside `tdps_routes`.

### Event Tracking

**New event type** — Add `PdfOpenEvent` to `event_processing`, following the exact pattern of `PaperOpenEvent`:

```rust
pub struct PdfOpenEvent {
    pub paper_id: String,
}
```

Add `PdfOpen(PdfOpenEvent)` variant to the `Event` enum with event name `"pdf_open"`.

**New route** — `POST /api/papers/{id}/pdf-open` in `web/src/routes/papers.rs`:

- Fire-and-forget dispatch (returns `204 No Content`, no `api` crate handler needed)
- Mirrors the existing `paper_open_handler` pattern

**Frontend fires** the POST when the user clicks the PDF button (fire-and-forget, does not block the PDF from opening).

### Frontend

**Paper page** (`paper/[id]/+page.svelte`):

- Replace the disabled "Coming soon" button with an enabled `<a>` linking to `/pdfs/{lyti}.pdf`
- `target="_blank"` to open in a new browser tab
- Placed at the top of the paper content area
- Styled as a button matching the existing design language
- On click, fire `POST /api/papers/{lyti}/pdf-open` (fire-and-forget, does not prevent navigation)

**Vite dev proxy** (`vite.config.ts`):

- Add `/pdfs` proxy entry alongside `/tdps` and `/api`, targeting `http://localhost:50000`

### Docker

**config.docker.toml** — Add:

```toml
tdps_pdf_root = "/data/tdps_pdf"
```

**docker-compose.yml** — Add volume mount to web service:

```yaml
- ${TDP_PDF_ROOT:?Set TDP_PDF_ROOT in .env}:/data/tdps_pdf:ro
```

**.env.example** — Add:

```
TDP_PDF_ROOT=/path/to/tdps_pdf
```

### Documentation

**config.toml.example** — Add `tdps_pdf_root` field.

**CLAUDE.md** — Update the config example in "Local Setup Prerequisites" to include `tdps_pdf_root`.

## Files Changed

| File | Change |
|------|--------|
| `data_processing/src/config.rs` | Add `tdps_pdf_root: String` |
| `web/src/state.rs` | Add `tdps_pdf_root: String` to `AppState` |
| `web/src/main.rs` | Pass `tdps_pdf_root` to `AppState::new()` |
| `web/src/routes/pdfs.rs` | New file — `serve_pdf_file` handler |
| `web/src/routes/papers.rs` | Add `pdf_open_handler` (fire-and-forget event dispatch) |
| `web/src/routes/mod.rs` | Register `/pdfs/{*path}` route, `POST /api/papers/{id}/pdf-open`, add `mod pdfs` |
| `event_processing/src/lib.rs` | Add `PdfOpenEvent` struct and `PdfOpen` variant to `Event` enum |
| `frontend/src/routes/paper/[id]/+page.svelte` | Replace disabled button with live PDF link at top, fire pdf-open event on click |
| `frontend/vite.config.ts` | Add `/pdfs` proxy |
| `config.toml.example` | Add `tdps_pdf_root` |
| `config.docker.toml` | Add `tdps_pdf_root` |
| `docker-compose.yml` | Add PDF volume mount to web service |
| `.env.example` | Add `TDP_PDF_ROOT` |
| `CLAUDE.md` | Update config example |
