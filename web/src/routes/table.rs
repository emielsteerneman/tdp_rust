use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_table_handler(
    State(state): State<AppState>,
    Path((lyti, content_seq)): Path<(String, u32)>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let args = api::get_table::GetTableArgs {
        paper: lyti,
        content_seq,
    };
    let result = api::get_table::get_table(
        state.metadata_client.clone(),
        args,
        state.activity_client.clone(),
        api::activity::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
