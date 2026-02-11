use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_papers_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<data_structures::file::TDPName>>>, ApiError> {
    let papers = api::list_papers::list_papers(state.metadata_client.clone())
        .await
        .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(papers)))
}

pub async fn get_paper_handler(
    State(state): State<AppState>,
    Path(lyti): Path<String>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Parse lyti string into TDPName
    let tdp_name = data_structures::file::TDPName::try_from(lyti.as_str())
        .map_err(|e| ApiError::bad_request(format!("Invalid paper ID: {}", e)))?;

    // Get the markdown content
    let markdown = state
        .metadata_client
        .get_tdp_markdown(tdp_name)
        .await
        .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(ApiResponse::new(markdown)))
}
