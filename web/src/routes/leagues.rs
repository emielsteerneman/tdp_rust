use axum::extract::State;
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_leagues_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<data_structures::file::League>>>, ApiError> {
    let leagues = api::list_leagues::list_leagues(
        state.metadata_client.clone(),
        state.activity_client.clone(),
        api::activity::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(leagues)))
}
