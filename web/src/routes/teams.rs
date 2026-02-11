use axum::extract::{Query, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_teams_handler(
    State(state): State<AppState>,
    Query(args): Query<api::list_teams::ListTeamsArgs>,
) -> Result<Json<ApiResponse<Vec<data_structures::file::TeamName>>>, ApiError> {
    let teams = api::list_teams::list_teams(state.metadata_client.clone(), args)
        .await
        .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(teams)))
}
