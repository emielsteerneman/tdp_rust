use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;
use data_structures::content::PaperInfo;

pub async fn get_paper_info_handler(
    State(state): State<AppState>,
    Path(lyti): Path<String>,
) -> Result<Json<ApiResponse<PaperInfo>>, ApiError> {
    let args = api::get_paper_info::GetPaperInfoArgs { paper: lyti };
    let result = api::get_paper_info::get_paper_info(
        state.metadata_client.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(result)))
}
