use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_image_handler(
    State(state): State<AppState>,
    Path((paper_lyt, content_seq)): Path<(String, u32)>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let args = api::get_image::GetImageArgs {
        paper: paper_lyt,
        content_seq,
    };
    let result = api::get_image::get_image(
        state.metadata_client.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
