mod abstract_text;
mod references;
mod image;
mod leagues;
mod paper_info;
mod papers;
mod paragraph;
mod pdfs;
mod search;
mod table;
mod table_of_contents;
mod tdps;
mod suggestion;
mod registry;
mod teams;
mod years;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api_routes = Router::new()
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
        .with_state(state.clone());

    // PDF file routes (no activity logging, static file serving)
    let pdfs_routes = Router::new()
        .route("/pdfs/{*path}", get(pdfs::serve_pdf_file))
        .with_state(state);

    // Serve static frontend files with SPA fallback
    // If a file exists, serve it; otherwise serve index.html for client-side routing
    let static_files = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // Combine API routes with static file serving
    // API routes take precedence, then static files
    Router::new()
        .merge(api_routes)
        .merge(tdps_routes)
        .merge(pdfs_routes)
        .fallback_service(static_files)
        .layer(cors)
}
