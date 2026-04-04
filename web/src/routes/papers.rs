use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
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
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(papers)))
}

pub async fn paper_open_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
    headers: HeaderMap,
) -> StatusCode {
    let referrer = headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::PaperOpen(event_processing::PaperOpenEvent {
            paper_id: paper_lyt,
            referrer,
        }),
    );

    StatusCode::NO_CONTENT
}

pub async fn pdf_open_handler(
    State(state): State<AppState>,
    Path(paper_lyt): Path<String>,
) -> StatusCode {
    state.dispatcher.dispatch(
        event_processing::EventSource::Web,
        event_processing::Event::PdfOpen(event_processing::PdfOpenEvent {
            paper_id: paper_lyt,
        }),
    );

    StatusCode::NO_CONTENT
}
