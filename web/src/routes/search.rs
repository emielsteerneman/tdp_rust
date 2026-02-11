use axum::extract::{Query, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn search_handler(
    State(state): State<AppState>,
    Query(args): Query<api::search::SearchArgs>,
) -> Result<Json<ApiResponse<data_structures::intermediate::SearchResult>>, ApiError> {
    let result = api::search::search_structured(&state.searcher, args)
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(ApiResponse::new(result)))
}
