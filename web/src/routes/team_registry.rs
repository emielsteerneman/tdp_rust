use axum::extract::{Path, State};
use axum::Json;

use crate::dto::ApiResponse;
use crate::error::ApiError;
use crate::state::AppState;
use data_access::teams::TeamMetadataEntry;

pub async fn get_team_info_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<Vec<TeamMetadataEntry>>>, ApiError> {
    let registry = state.team_registry.as_ref()
        .ok_or_else(|| ApiError::not_found("Team registry not configured"))?;

    let args = api::get_team_info::GetTeamInfoArgs { team: name };

    let entries = api::get_team_info::get_team_info(
        registry.clone(),
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| ApiError::internal_server_error(e.to_string()))?;

    Ok(Json(ApiResponse::new(entries)))
}

pub async fn update_team_info_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(args): Json<api::update_team_info::UpdateTeamInfoArgs>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let registry = state.team_registry.as_ref()
        .ok_or_else(|| ApiError::not_found("Team registry not configured"))?;

    let result = api::update_team_info::update_team_info(
        registry.clone(),
        &name,
        args,
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await
    .map_err(|e| match e {
        api::error::ApiError::Forbidden(_) => ApiError::forbidden(e.to_string()),
        api::error::ApiError::Argument(_, _) => ApiError::bad_request(e.to_string()),
        api::error::ApiError::Internal(_) => ApiError::internal_server_error(e.to_string()),
    })?;

    Ok(Json(ApiResponse::new(result)))
}
