use axum::extract::State;
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn submit_suggestion_handler(
    State(state): State<AppState>,
    Json(args): Json<api::suggestion::SuggestionArgs>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let result = api::suggestion::submit_suggestion(
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
