use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_references_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let args = api::get_references::GetReferencesArgs { paper: paper_lyt };
    let result = api::get_references::get_references(
        state.metadata_client.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
