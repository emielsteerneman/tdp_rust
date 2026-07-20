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

use std::path::{Path, PathBuf};

use axum::handler::HandlerWithoutStateExt;
use axum::http::{header, StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::state::AppState;

const STATIC_ROOT: &str = "static";

/// Map an extension-less URL path to its prerendered `{root}/{path}.html` candidate.
///
/// Returns `None` (i.e. "no prerendered file, use the SPA shell") for the site root,
/// paths that already carry a file extension, and any path containing `..` (traversal
/// guard). Existence of the returned candidate is NOT checked here — that happens at
/// serve time.
fn resolve_prerendered(root: &Path, url_path: &str) -> Option<PathBuf> {
    // Traversal guard: never let a request escape the static root.
    if url_path.contains("..") {
        return None;
    }
    let trimmed = url_path.trim_start_matches('/');
    // Root ("" / "/") is served by ServeDir's directory index (index.html).
    if trimmed.is_empty() {
        return None;
    }
    // A path whose final segment already has an extension is a concrete asset request,
    // not an extension-less content route.
    let last_segment = trimmed.rsplit('/').next().unwrap_or("");
    if last_segment.contains('.') {
        return None;
    }
    Some(root.join(format!("{trimmed}.html")))
}

fn html_response(bytes: Vec<u8>) -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        bytes,
    )
        .into_response()
}

/// Fallback for requests ServeDir could not satisfy with an exact/dir match.
///
/// 1. Extension-less path with a matching prerendered `static/{path}.html` -> serve it.
/// 2. Otherwise serve the SPA shell `static/200.html` (client-side routing).
/// 3. If even that is missing -> 404.
async fn prerender_fallback(uri: Uri) -> Response {
    let root = Path::new(STATIC_ROOT);

    if let Some(candidate) = resolve_prerendered(root, uri.path()) {
        if let Ok(bytes) = tokio::fs::read(&candidate).await {
            return html_response(bytes);
        }
    }

    match tokio::fs::read(root.join("200.html")).await {
        Ok(bytes) => html_response(bytes),
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

    // Serve static frontend files. ServeDir handles exact-file and directory-index
    // matches (e.g. `/_app/...`, `/robots.txt`, `/` -> index.html). Anything it can't
    // resolve falls through to `prerender_fallback`, which maps extension-less content
    // URLs to their prerendered `.html` and otherwise serves the SPA shell `200.html`.
    let static_files =
        ServeDir::new(STATIC_ROOT).not_found_service(prerender_fallback.into_service());

    // Combine API routes with static file serving
    // API routes take precedence, then static files
    Router::new()
        .merge(api_routes)
        .merge(tdps_routes)
        .merge(pdfs_routes)
        .fallback_service(static_files)
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::resolve_prerendered;
    use std::path::Path;

    #[test]
    fn resolves_extensionless_paper_path() {
        let root = Path::new("static");
        assert_eq!(
            resolve_prerendered(root, "/paper/soccer_smallsize__2024__RoboTeam_Twente"),
            Some(root.join("paper/soccer_smallsize__2024__RoboTeam_Twente.html"))
        );
    }

    #[test]
    fn root_has_no_prerendered_candidate() {
        assert_eq!(resolve_prerendered(Path::new("static"), "/"), None);
    }

    #[test]
    fn resolves_top_level_route() {
        // Existence is checked at serve time, not here.
        let root = Path::new("static");
        assert_eq!(
            resolve_prerendered(root, "/search"),
            Some(root.join("search.html"))
        );
    }

    #[test]
    fn rejects_path_traversal() {
        assert_eq!(
            resolve_prerendered(Path::new("static"), "/../etc/passwd"),
            None
        );
    }

    #[test]
    fn ignores_paths_with_extension() {
        assert_eq!(
            resolve_prerendered(Path::new("static"), "/og-default.png"),
            None
        );
    }
}
