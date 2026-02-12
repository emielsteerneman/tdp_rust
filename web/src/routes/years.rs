use axum::extract::State;
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;

pub async fn list_years_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<u32>>>, ApiError> {
    let years = api::list_years::list_years(
        state.metadata_client.clone(),
        api::paper_filter::PaperFilter::default(),
    )
    .await
    .map_err(|e| ApiError::from(e))?;

    Ok(Json(ApiResponse::new(years)))
}
