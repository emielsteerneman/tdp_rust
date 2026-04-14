## Purpose
Axum HTTP server. Thin wrapper — all logic lives in `api`. Serves the frontend SPA.

## Structure
- `routes/mod.rs` — all route registration. API routes get activity-logging middleware.
- `state.rs` — `AppState` with Arc'd clients + `tdps_markdown_root` + `tdps_pdf_root` + optional `registry`.
- `middleware.rs` — logs HTTP requests as `HttpRequestEvent` via the event dispatcher.
- `error.rs` — `ApiError` struct implementing `IntoResponse` for JSON error responses.
- `dto.rs` — `ApiResponse<T>` wrapper (all responses have a `data` field).

## Adding a New Route
1. Create `routes/<name>.rs` with handler function.
2. Extract state via `State(state): State<AppState>`, args via `Query`/`Path`/`Json`.
3. Call `api::handler::handler_name()` with `EventSource::Web`.
4. Return `Ok(Json(ApiResponse::new(result)))`.
5. Register in `routes/mod.rs`.

## API Discoverability
- `GET /api` — returns JSON list of all API routes with methods and descriptions.

## Port
Web server runs on `:50000`.

## Static Serving
- `/api/*` — API routes (with activity logging middleware)
- `/tdps/{*path}` — serves TDP markdown files from `tdps_markdown_root` (with path traversal protection)
- `/pdfs/{*path}` — serves original PDF files from `tdps_pdf_root`
- Everything else — falls back to `static/index.html` (SPA routing)
