mod abstract_text;
mod api_index;
mod references;
mod image;
mod leagues;
mod paper_info;
mod papers;
mod paragraph;
mod pdfs;
mod search;
mod sitemap;
mod table;
mod table_of_contents;
mod tdps;
mod suggestion;
mod registry;
mod teams;
mod years;

use std::path::Path;

use axum::http::{header, StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

const STATIC_ROOT: &str = "static";

/// Best-effort `Content-Type` from a path's extension, covering the asset types a
/// SvelteKit static build emits. Unknown extensions fall back to octet-stream.
fn content_type_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "webmanifest" => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

fn file_response(bytes: Vec<u8>, content_type: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type)], bytes).into_response()
}

/// Relative `{path}.html` an extension-less content URL should resolve to, or `None`
/// for the site root, paths that already carry a file extension, and traversal attempts.
/// Existence is checked by the caller.
fn page_html_path(url_path: &str) -> Option<String> {
    if url_path.contains("..") {
        return None;
    }
    let trimmed = url_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let last_segment = trimmed.rsplit('/').next().unwrap_or("");
    if last_segment.contains('.') {
        return None;
    }
    Some(format!("{trimmed}.html"))
}

/// Serve the SvelteKit static build directly (no `ServeDir`), so we fully control
/// status codes and never emit a directory 307-redirect. Prerendered paper pages sit
/// beside a companion `paper/{id}/` directory (holding `__data.json` for client-side
/// navigation); `ServeDir` would 307 `/paper/{id}` to `/paper/{id}/` and never serve
/// the page, and its `not_found_service` responses come back as 404. This handler
/// avoids both.
///
/// Resolution order (all hits are 200): exact file (assets, root files, `__data.json`)
/// -> extension-less `{path}.html` prerendered page -> site root `index.html` -> SPA
/// shell `200.html`. Only a missing `200.html` yields 404.
async fn serve_static(uri: Uri) -> Response {
    let raw = uri.path();
    if raw.contains("..") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let root = Path::new(STATIC_ROOT);
    let rel = raw.trim_start_matches('/');

    // 1. Exact file: assets (/_app/...), root files (/robots.txt, /favicon.png), and
    //    client-nav data (/paper/{id}/__data.json). read() errors on a directory, so a
    //    request for the extension-less `/paper/{id}` falls through to step 2.
    if !rel.is_empty() {
        if let Ok(bytes) = tokio::fs::read(root.join(rel)).await {
            return file_response(bytes, content_type_for(rel));
        }
    }

    // 2. Extension-less content route -> prerendered `{path}.html`.
    if let Some(html) = page_html_path(raw) {
        if let Ok(bytes) = tokio::fs::read(root.join(&html)).await {
            return file_response(bytes, "text/html; charset=utf-8");
        }
    }

    // 3. Site root -> index.html.
    if rel.is_empty() {
        if let Ok(bytes) = tokio::fs::read(root.join("index.html")).await {
            return file_response(bytes, "text/html; charset=utf-8");
        }
    }

    // 4. SPA shell for any other (dynamic / unknown) route.
    match tokio::fs::read(root.join("200.html")).await {
        Ok(bytes) => file_response(bytes, "text/html; charset=utf-8"),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
        .route("/api", get(api_index::api_index_handler))
        .route("/api/search", get(search::search_handler))
        .route("/api/papers", get(papers::list_papers_handler))
        .route("/api/papers/{id}/open", post(papers::paper_open_handler))
        .route("/api/papers/{id}/pdf-open", post(papers::pdf_open_handler))
        .route("/api/papers/{id}/toc", get(table_of_contents::get_table_of_contents_handler))
        .route("/api/papers/{id}/paragraph/{seq}", get(paragraph::get_paragraph_handler))
        .route("/api/papers/{id}/table/{seq}", get(table::get_table_handler))
        .route("/api/papers/{id}/image/{seq}", get(image::get_image_handler))
        .route("/api/papers/{id}/abstract", get(abstract_text::get_abstract_handler))
        .route("/api/papers/{id}/references", get(references::get_references_handler))
        .route("/api/papers/{id}/info", get(paper_info::get_paper_info_handler))
        .route("/api/teams", get(teams::list_teams_handler))
        .route("/api/leagues", get(leagues::list_leagues_handler))
        .route("/api/years", get(years::list_years_handler))
        .route("/api/suggestion", post(suggestion::submit_suggestion_handler))
        .route("/api/registry/team/{name}", get(registry::get_team_info_handler))
        .route("/api/registry/team", post(registry::update_team_info_handler))
        .route("/api/registry/league/{name}", get(registry::get_league_info_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::activity_logging,
        ))
        .with_state(state.clone());

    // TDP file routes (no activity logging, static file serving)
    let tdps_routes = Router::new()
        .route("/tdps/{*path}", get(tdps::serve_tdps_file))
        .route("/sitemap.xml", get(sitemap::sitemap_handler))
        .with_state(state.clone());

    // PDF file routes (no activity logging, static file serving)
    let pdfs_routes = Router::new()
        .route("/pdfs/{*path}", get(pdfs::serve_pdf_file))
        .with_state(state);

    // API/tdps/pdfs routes take precedence; everything else is served from the static
    // build by `serve_static` (prerendered pages, assets, SPA shell).
    Router::new()
        .merge(api_routes)
        .merge(tdps_routes)
        .merge(pdfs_routes)
        .fallback(serve_static)
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::{content_type_for, page_html_path};

    #[test]
    fn page_path_for_extensionless_paper() {
        assert_eq!(
            page_html_path("/paper/soccer_smallsize__2024__RoboTeam_Twente").as_deref(),
            Some("paper/soccer_smallsize__2024__RoboTeam_Twente.html")
        );
    }

    #[test]
    fn page_path_none_for_root() {
        assert_eq!(page_html_path("/"), None);
    }

    #[test]
    fn page_path_for_top_level_route() {
        // Existence is checked at serve time, not here.
        assert_eq!(page_html_path("/search").as_deref(), Some("search.html"));
    }

    #[test]
    fn page_path_rejects_traversal() {
        assert_eq!(page_html_path("/../etc/passwd"), None);
    }

    #[test]
    fn page_path_none_for_asset_with_extension() {
        assert_eq!(page_html_path("/og-default.png"), None);
    }

    #[test]
    fn content_types_cover_sveltekit_assets() {
        assert_eq!(content_type_for("app.js"), "text/javascript; charset=utf-8");
        assert_eq!(content_type_for("style.css"), "text/css; charset=utf-8");
        assert_eq!(content_type_for("paper/x/__data.json"), "application/json");
        assert_eq!(content_type_for("og-default.png"), "image/png");
        assert_eq!(content_type_for("page.html"), "text/html; charset=utf-8");
        assert_eq!(content_type_for("noext"), "application/octet-stream");
    }
}
