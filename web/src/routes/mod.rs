mod leagues;
mod papers;
mod search;
mod teams;
mod years;

use axum::middleware;
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/search", get(search::search_handler))
        .route("/api/papers", get(papers::list_papers_handler))
        .route("/api/papers/{id}", get(papers::get_paper_handler))
        .route("/api/teams", get(teams::list_teams_handler))
        .route("/api/leagues", get(leagues::list_leagues_handler))
        .route("/api/years", get(years::list_years_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::activity_logging,
        ))
        .with_state(state)
        .layer(cors)
}
