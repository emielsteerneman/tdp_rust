use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn get_table_of_contents_handler(
    State(state): State<AppState>,
    Path(lyti): Path<String>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let args = api::get_table_of_contents::GetTableOfContentsArgs { paper: lyti };
    let result = api::get_table_of_contents::get_table_of_contents(
        state.metadata_client.clone(),
        args,
        state.activity_client.clone(),
        api::activity::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
