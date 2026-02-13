use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_papers_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<data_structures::file::TDPName>>>, ApiError> {
    let papers = api::list_papers::list_papers(
        state.metadata_client.clone(),
        api::paper_filter::PaperFilter::default(),
        state.activity_client.clone(),
        api::activity::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(papers)))
}

pub async fn get_paper_handler(
    State(state): State<AppState>,
    Path(lyti): Path<String>,
    headers: HeaderMap,
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

    let referrer = headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    api::activity::log_activity(
        state.activity_client.clone(),
        api::activity::EventSource::Web,
        "paper_open",
        serde_json::json!({
            "paper_id": lyti,
            "referrer": referrer,
        }),
    );

    Ok(Json(ApiResponse::new(markdown)))
}
